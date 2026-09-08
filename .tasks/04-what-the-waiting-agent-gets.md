# 04. What the waiting agent gets

## What to build

The Response as the agent reads it. Each Answer with files carries an
`attachments` list — the field the schema gained in task 02 — holding the
path the session reads each file at, in the order they were attached: the
bind path on Linux and the directory's own real path where nothing is
mounted, the same answer the prompt's listing gives. The server fills it from
the record's rows on the way out, on a waited Response and on a fetched one
alike, so `verkstead ask` and `verkstead answers` print the same thing; the
CLI passes the field through as it passes the rest. An Answer with no files
carries no list, and a stored Response written before the field existed
reads as it always did.

The sandbox binds the Conversation's attachments directory at every launch,
making it empty where it is missing, rather than only where a file is already
in it — so a session blocked on an ask finds the file the Response names.
The prompt's listing still says nothing for a Conversation with nothing
attached: an empty directory nobody is told about is not a path an agent is
told about and finds empty.

## Acceptance criteria

- [ ] A Response to a Set with files on it, waited for or fetched, lists each
      Answer's files by the path the session reads them at, and one without
      lists nothing.
- [ ] A session launched with nothing attached can read a file put on an
      Answer after it started, on each platform's arrangement.
- [ ] A stored Response from before the field existed reads, and the CLI
      prints the field unchanged.
