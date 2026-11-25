kind: feat

- A card whose session has not run for 6 hours leaves the sidebar by itself, as `x` would
  remove it: an exited card counted from its exit, a stopped one from its conversation's last
  change. `tidy_after` in the config file, or `HARBOUR_TIDY_AFTER`, sets the hours (`"12h"`) or
  turns it `"off"`, and `claude --resume` still finds the conversation.
