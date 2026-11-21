use crate::text::{clean, columns};
use crate::zoom::Headline;

pub const OPTIONS: [&str; 7] = [
    "--delimiter=\t",
    "--with-nth=2..",
    "--nth=2,3",
    "--tabstop=1",
    "--no-sort",
    "--layout=reverse",
    "--bind=enter:accept",
];

pub fn lines(headlines: &[Headline]) -> Vec<u8> {
    let titles: Vec<String> = headlines.iter().map(|line| clean(&line.title)).collect();
    let places: Vec<String> = headlines.iter().map(|line| clean(&line.place)).collect();
    let ages: Vec<&str> = headlines
        .iter()
        .map(|line| line.age.as_deref().unwrap_or_default())
        .collect();
    let widest = |texts: &[String]| texts.iter().map(|text| columns(text)).max().unwrap_or(0);
    let aged = ages.iter().map(|age| columns(age)).max().unwrap_or(0);
    let titled = widest(&titles);
    let placed = widest(&places);
    let mut listed = String::new();
    for (index, headline) in headlines.iter().enumerate() {
        listed.push_str(&format!(
            "{index}\t{}\t{} \t{} \t{}{}\n",
            headline.state.glyph(),
            padded(&titles[index], titled),
            padded(&places[index], placed),
            " ".repeat(aged - columns(ages[index])),
            ages[index],
        ));
    }
    listed.into_bytes()
}

pub fn chosen(line: &[u8]) -> Option<usize> {
    let number = line.split(|byte| *byte == b'\t').next()?;
    std::str::from_utf8(number).ok()?.parse().ok()
}

fn padded(text: &str, width: usize) -> String {
    format!("{text}{}", " ".repeat(width.saturating_sub(columns(text))))
}
