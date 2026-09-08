# 01. Record a file on an Answer

## What to build

The store's second origin and the endpoints that use it: a file put on a
waiting Question Set under one of its labels, and taken off again.

The attachments rows gain the Set they were put on and the Question label,
added the way earlier columns were added to a STRICT table — a migration that
looks for the column and adds it once. The Brief's rows carry neither. The
stored origin word is `answer`, and the wire's origin gains the matching
value; reading an unknown word stays the refusal it is today.

Two endpoints beside the Brief's, addressed by the Set rather than by the
Conversation: put a file on a Set under a label, the raw bytes as the body and
the name in the path as the Brief's upload has it, and remove one by the
row's own id. The file lands in the Conversation's one flat directory with the
same cap, the same plain-name rule and the same counting-up of a name already
taken, so a file named like one of the Brief's becomes `name-2.ext`. The row
and the bytes are written and taken away together, in the order the Brief's
are.

Refused, each by name, where the Set is answered, locked, or its Conversation
is Closed — the freeze the Brief has at the moment the work starts, moved to
the moment the Set settles — and where the label is not one the Set asks: a
Heading, a label the Set does not carry, or a Set that cannot be read. A Set
that is a Deferred Ask takes files like any other.

The Set view carries the files put on it: a list of rows, each with the row's
id, name, size, origin and the label it was put under, in the order they were
attached. This is what the sheet draws from in task 02 and the record from in
task 03, so it is on the view whether or not the Set has settled.

CONTEXT.md's Attachment entry says there are two origins now, what an Answer's
file is keyed by, and when it freezes.

## Acceptance criteria

- [ ] A file put on a waiting Set under one of its labels is on disk in the
      Conversation's directory and in the record with origin `answer`, the Set
      and the label, and comes back on the Set view under that label; a second
      of the same name counts up.
- [ ] The upload and the removal are each refused by name on an answered Set,
      a locked Set, a Set of a Closed Conversation, and a label the Set does
      not ask, and leave nothing on disk.
- [ ] A record written before the columns existed opens, its Brief rows read
      as the Brief's, and the server suite covers the migration.
