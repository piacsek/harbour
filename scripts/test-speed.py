import sys
import xml.etree.ElementTree as ElementTree
from xml.sax.saxutils import escape

SHOWN = 15
WIDTH = 760
MARGIN = 16
ROW = 36
TOP = 44
LABEL_CHARS = 90
STYLE = """<style>
text { font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; fill: #1f2328; }
.title { font-size: 15px; font-weight: 600; }
.note { fill: #59636e; }
rect { fill: #0969da; }
@media (prefers-color-scheme: dark) {
  text { fill: #e6edf3; }
  .note { fill: #9198a1; }
  rect { fill: #4493f8; }
}
</style>"""


def cases(paths):
    found = []
    for path in paths:
        for case in ElementTree.parse(path).iter("testcase"):
            binary = case.get("classname", "").split("::")[-1]
            label = f"{binary} {case.get('name', '')}".strip()
            found.append((float(case.get("time", "0")), label))
    return found


def shortened(label):
    if len(label) <= LABEL_CHARS:
        return label
    return label[: LABEL_CHARS - 1] + "…"


def chart(found):
    slowest = sorted(found, key=lambda case: -case[0])[:SHOWN]
    longest = slowest[0][0] or 1
    room = WIDTH - 2 * MARGIN
    height = TOP + ROW * len(slowest) + 28
    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{height}" '
        f'viewBox="0 0 {WIDTH} {height}" role="img" aria-label="The slowest tests">',
        STYLE,
        f'<text class="title" x="{MARGIN}" y="24">Slowest tests</text>',
    ]
    for row, (seconds, label) in enumerate(slowest):
        y = TOP + ROW * row
        bar = max(2, round(room * seconds / longest))
        lines.append(f'<text x="{MARGIN}" y="{y + 12}">{escape(shortened(label))}</text>')
        lines.append(
            f'<text x="{WIDTH - MARGIN}" y="{y + 12}" text-anchor="end">{seconds:.2f} s</text>'
        )
        lines.append(f'<rect x="{MARGIN}" y="{y + 17}" width="{bar}" height="8" rx="2"/>')
    total = sum(seconds for seconds, _ in found)
    lines.append(
        f'<text class="note" x="{MARGIN}" y="{height - 12}">'
        f"{len(found)} tests, {total:.1f} s in all</text>"
    )
    lines.append("</svg>")
    return "\n".join(lines) + "\n"


def main(paths):
    if not paths:
        print("usage: test-speed.py <junit.xml>...", file=sys.stderr)
        return 1
    try:
        found = cases(paths)
    except (OSError, ElementTree.ParseError, ValueError) as error:
        print(f"test-speed: cannot read the reports: {error}", file=sys.stderr)
        return 1
    if not found:
        print("test-speed: no test in " + ", ".join(paths), file=sys.stderr)
        return 1
    sys.stdout.write(chart(found))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
