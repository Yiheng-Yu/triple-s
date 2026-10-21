Describes how responses are stored for a question of type multiple.
Spread corresponds to the optional spread element within a variable in
Triple-S XML, or the SPREAD line in Triple-S classic.

Multiple-choice responses use one of two layouts, depending on whether
spread is defined.

Without spread (default)
------------------------------

When spread is absent (None), each character in the question's data field
represents a category: 1 means selected and 0 means not selected.

Positions start at 1 and correspond directly to category codes:

- Category codes: 1, 2, 3, 4, 5
- Selection flags: 1, 0, 1, 0, 0 (stored as "10100")
- Selected codes: [1, 3]

With spread
--------------

When spread is present, the data contains consecutive slots called
subfields. The subfield count specifies the number of slots; width
specifies the number of characters in each slot.

Width can be omitted for fixed-width (.asc) data when equal-width slots
exactly fill the question's field. It is then calculated as the field
width divided by the subfield count. Width is required for CSV data.

Examples
--------

Two subfields, each one character wide:

- Raw data: "13"
- Split: "1" | "3"
- Selected codes: [1, 3]

One subfield, two characters wide:

- Raw data: "13"
- Split: "13"
- Selected codes: [13]

Two subfields, each two characters wide. The second slot contains two
spaces and is unused:

- Raw data: "13  "
- Split: "13" | "  "
- Selected codes: [13]

Three subfields, each three characters wide:

- Raw data: "001042999"
- Split: "001" | "042" | "999"
- Selected codes: [1, 42, 999]

The subfield count and width determine how to split the data; the digits
alone do not determine where one category code ends and the next begins.
