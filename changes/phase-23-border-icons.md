kind: feat

- `nerd_font = true` in the config file puts an icon before `claude` and before `nvim` in the
  pane's border. It needs a terminal font with Nerd Font glyphs, so it is off by default, and an
  editor other than Neovim gets no icon. `harbour config` shows the setting and where it comes
  from, and a value other than `true` or `false` is refused like any other bad setting.
