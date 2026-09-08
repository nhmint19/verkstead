# 02. The paperclip on the sheet

## What to build

Attaching from the answer sheet, the way the human specified it. Every
Question and Sub-question's free-text field carries a paperclip: an icon-only
press with no border, in a subdued text colour, sitting inside the field's
padding anchored to the bottom right, and vertically centred when the field
is a single line. A Heading has no field and no paperclip. The Question's
whole card takes a drop, the way the composer's whole box does, highlighted
while a drag carrying files is over it; folders are skipped without a word.
The set-level comment box takes nothing.

Reuse the one attaching piece the composers share rather than writing a
second. What differs here is what becomes of a chosen file: it goes up at
once, onto the Set under the Question's label through task 01's endpoint,
and its pill — dimmed until the record arrives, then the record's — is drawn
under the field with a × that removes it. The pills are read off the Set
view, so a reload keeps them beside the answers the device kept; nothing
about a file is held in the page. A refused upload is said on the sheet the
way the composer says one.

A Question with a file on it and nothing typed or picked is an Answer. The
sheet submits it as answered rather than marking it Unanswered, and the
server accepts it: the record's rows are the truth, and where a submission
is validated the server counts a label's files as an Answer to it. The
stored Response body stays what the human sent; the schema's Answer gains an
`attachments` list, default empty and left out when empty, so an older body
still reads — that list is filled from the rows wherever a Response is read
out, which tasks 04 and 05 use.

## Acceptance criteria

- [ ] Every answerable field on the sheet draws the paperclip as specified,
      a Heading draws none, and a pick or a drop on the card puts the file on
      the Set under that label and draws its pill under the field.
- [ ] A reload of the sheet draws the pills again from the record, and the ×
      removes the file from the record and the row.
- [ ] A Question with a file and nothing else is submitted as answered, the
      server accepts it, and one with neither a file nor anything typed or
      picked still goes back Unanswered.
