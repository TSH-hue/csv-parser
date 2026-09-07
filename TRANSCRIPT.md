# When a Comma Is Not a Separator

Complete spoken transcript. Code excerpts are linked to the runnable source.

## 00:00:00 — Two cells. One troublesome comma.

Mira has a shopping note: bread, milk. We want to import her name and her note into these two cells.

The text above is C S V: comma separated values. Each cell is a field. A row of related fields is a record.

Let's try the obvious solution. Split the text whenever we see a comma. There. One piece, two pieces, three pieces.

Wait, three? The note has been cut in half. Bread and milk were supposed to stay together in one cell.

Look at the two commas. Same character. Different jobs. The first separates fields. The second belongs to the note.

The quotes tell us which is which. They surround a field whose contents are allowed to include a comma.

Our job is to recover the two original values, keeping the comma in the note and leaving out the surrounding quotes.

## 00:00:59 — Build the decisions, then keep them working.

We'll build that reader together in Rust, one working case at a time. You can follow the whole film without typing.

First, we'll make fields and records. Then we'll see what quotes change, including a quote that belongs inside the text.

Finally, the input will arrive in pieces. We'll keep the same answers even when a piece ends at an awkward spot.

Every time we need a new decision, we'll add the small piece of Rust that makes it, and run the same example.

The source links include working checkpoints. Pause at a checkpoint if you'd like to build along; the next explanation won't depend on doing homework.

This reader has a stated set of rules. It reads UTF eight text, accepts line feed or carriage return plus line feed between records, and reports malformed quoting.

We'll introduce those details when they change a decision. For now, let's earn our first field with just four letters.

## 00:02:07 — Build one field.

Start with Mira. No comma yet. The answer is one field containing the four letters we can already see.

But the program receives a sequence, so it needs somewhere to keep the part it has read. Start an empty field.

Take M. Keep it. Take i, then r, then a. Each new byte joins the same unfinished field.

This becomes a vector of bytes in Rust. A vector can grow. Each push appends one byte to its end.

The loop borrows the input slice and visits its bytes. The little ampersand in the pattern gives us the byte value to push.

When the input really ends, the field is complete. These four bytes form the string Mira. Now we can return that value.

Try Noah instead. The letters changed, but the operation didn't. We accumulate content until something tells us the field is finished.

Code: [checkpoints/01_field.rs](checkpoints/01_field.rs), starting at line 5.

```rust
let mut field = Vec::new();
for &byte in input {
    field.push(byte);
}
```

## 00:03:08 — A separator finishes a field.

Now add a comma and tea. We want two fields in the same record. The comma itself is not part of either value.

Read Mira as before. When we reach the comma, the field has everything it needs. We can close that cell.

Put the finished string into the current record, then start an empty field for tea. Keep the record right here.

In Rust, the record is another vector. Its elements are strings, because it holds complete field values rather than individual bytes.

Memory take moves the current vector out and leaves an empty one behind. We can reuse the field variable without copying its whole contents.

Convert the bytes into a string, then push it into the record. The question mark passes any conversion error back to the caller.

At final end of input, finish tea as well. There was no comma after it, so the end itself supplies the finishing event.

Code: [checkpoints/02_records.rs](checkpoints/02_records.rs), starting at line 14.

```rust
let bytes = std::mem::take(&mut field);
let text = String::from_utf8(bytes)?;
row.push(text);
```

## 00:04:11 — A record boundary does two jobs.

Add a second shopper on a new line. L F labels the line-break byte on screen. Each shopper needs a separate record.

The first comma still finishes Mira. The line break arrives after tea, while tea is still in the field we are building.

So a record boundary first finishes that last field. Only then is the whole record ready to hand over.

Move the completed row out. Start an empty record. Noah and milk will go into that new record, not into Mira's.

These repeated actions deserve names: end field, and end record. Each name stands for one job we already understand visually.

Notice their boundary. End field doesn't decide whether a comma was allowed. It packages the field after the scanning logic makes that decision.

Replace tea with coffee. Nothing about record handling changes. Separate the decision about a boundary from the work performed when one is found.

Code: [rust/lib.rs](rust/lib.rs), starting at line 154.

```rust
b',' => self.end_field()?,
b'\n' => {
    self.end_field()?;
    return Ok(Some(self.end_record()));
}
b'\r' => {
```

## 00:05:18 — Empty is a value; absent is different.

What should this trailing comma mean? Mira has a second field, but there is no text in it. The answer includes an empty string.

The field builder is already empty. We still finish it when the record ends, so the empty cell keeps its place.

On the next line, the comma comes first. Same operation. Finish an empty first field, then read tea into the second.

A blank line represents one record containing one empty field. It has a record boundary, even though it has no field text.

A completely empty file is a different case. In this reader, it produces no records. We haven't started a record at all.

And a line break after a completed record does not create an extra imaginary row. There must be a new record to finish.

That's our plain-text checkpoint: content fills a field, a comma closes a field, and a record boundary closes the last field and the record.

Code: [rust/lib.rs](rust/lib.rs), starting at line 195.

```rust
    if !self.started {
        return Ok(None);
    }
    self.end_field()?;
    Ok(Some(self.end_record()))
}
```

## 00:06:26 — Inside the quotes, keep the comma.

Now return to the input that broke our split. Mira is ordinary. The next field starts with a quote.

Opening that quote changes how we read the field. The contour marks the part we're reading as quoted contents.

Bread goes into the field. Then comes the comma. We're still inside the quoted field, so this comma goes into the contents too.

At the closing quote, we stop reading quoted contents. The surrounding quotes describe the format; they aren't copied into the value.

We finish with two cells. Mira, and bread comma milk. The inner comma has reached the exact place it belongs.

Our first version can remember inside quotes with a boolean. The comma branch only separates fields when that boolean is false.

Remove the quotes around bread and milk, and the inner comma becomes a separator again. Punctuation gets its job from context.

Code: [checkpoints/03_quoted.rs](checkpoints/03_quoted.rs), starting at line 16.

```rust
} else if in_quotes {
    field.push(byte);
} else if byte == b',' {
    row.push(String::from_utf8(std::mem::take(&mut field))?);
} else if byte == b'\n' {
```

## 00:07:30 — One field can contain a line break.

Suppose the note has two lines: top, then bottom. That's still one shopper and one note, so it must remain one record.

The line break arrives while the field is quoted. Watch the cell grow vertically. Bottom belongs below top inside that same cell.

After the closing quote, another line break arrives. This one finishes the record. Same byte, different context again.

We don't need a special rule for every allowed content character. Inside quotes, an ordinary byte is simply added to the field.

That's a useful abstraction we've earned. Quoted content is one category. The only byte needing a new decision inside it is a quote.

A comma, a space, and a line break all travel through the content path here. Their shapes differ; their job is the same.

Our second working agreement is simple: inside quotes, preserve contents. Outside quotes, separators can finish fields and records. Now let's challenge the quote itself.

Code: [checkpoints/03_quoted.rs](checkpoints/03_quoted.rs), starting at line 16.

```rust
} else if in_quotes {
    field.push(byte);
} else if byte == b',' {
```

## 00:08:42 — Where may a quoted field begin?

Before adding escapes, pin down where quoting starts. In our rules, an opening quote must be the first byte of a field.

Here the comma finished Mira. The next field is empty and hasn't started. A quote here can open quoted contents.

But in tea quote time, we've already started a bare field. That quote cannot quietly change the interpretation of its earlier letters.

We report a quote in a bare field, at this source byte. The malformed current record won't be handed to the application.

Spaces also count as contents. A space before a quote has already started a bare field. This reader doesn't secretly trim it away.

These are format decisions, not guesses about what the writer intended. A stated rule gives us a clear result and a clear error.

A field beginning with two quotes is different: it can be an empty quoted field. There was no bare content before the opening quote.

Code: [checkpoints/04_lookahead.rs](checkpoints/04_lookahead.rs), starting at line 37.

```rust
} else if byte == b'"' {
    if !field.is_empty() {
        return Err("quote in bare field");
    }
    in_quotes = true;
} else {
```

## 00:09:48 — Two marks in the source, one in the value.

Noah's note is say, then hi in quotation marks. We want those inner quote characters to survive in the field value.

C S V writes a content quote as two consecutive quotes inside the quoted field. These two source marks represent one character.

Read the first mark of the pair. If we immediately declare the field closed, the second quote becomes a problem.

Instead, inspect the next byte. It's another quote. That tells us this pair belongs to the contents, not the field boundary.

Bring the two marks together. Append one quote. Then keep reading the same field. The h and i come next.

The pair after hi works exactly the same way. It adds the second content quote, the one after hi in the note.

One final quote remains. It closes the field. Three adjacent marks can be a doubled content quote followed by a closing quote.

Code: [checkpoints/04_lookahead.rs](checkpoints/04_lookahead.rs), starting at line 15.

```rust
if input.peek() == Some(&b'"') {
    input.next();
    field.push(b'"');
} else {
    in_quotes = false;
```

## 00:10:52 — The following byte resolves the question.

Let's slow down the decision at one quote. We've read tea inside a quoted field and now reached a quote mark.

If the following byte is another quote, keep one quote in the text and continue reading quoted contents.

If it's a comma, the quote we just saw was the closing quote. Finish tea, then begin the next field.

A record ending can finish the record. Final end of input can finish the last field. Those are legal ways to complete it.

But an x right after the closing quote has no job in this format. It is neither an escaped quote nor a separator.

Report that x as an unexpected character after a quote. We don't discard it, and we don't quietly merge it into tea.

So a quote doesn't answer the whole question by itself. The next byte decides which interpretation fits. That's the context we must preserve.

Code: [checkpoints/04_lookahead.rs](checkpoints/04_lookahead.rs), starting at line 35.

```rust
} else if closed_quote {
    return Err("character after closing quote");
```

## 00:11:53 — What if there is no next byte yet?

With the entire input in memory, our checkpoint can peek at the next byte. That lets us handle the doubled quote directly.

Peek observes the next byte without consuming it. If it is another quote, we consume that second mark and append one quote.

Run Noah's note. The pair before hi becomes one mark, the pair after hi becomes another, and the final mark closes the field.

Now put a delivery boundary between the two marks in the first pair. The current piece ends just after the first quote.

Peek has no byte to show. But the file hasn't necessarily ended. Its next piece may begin with exactly the quote we need.

We can't decide that the field is closed just because this piece is empty. And we can't forget the letters already collected.

The complete-input version has taught us the decision. To stream, we need to preserve an unanswered decision until more input arrives.

Code: [checkpoints/04_lookahead.rs](checkpoints/04_lookahead.rs), starting at line 15.

```rust
    if input.peek() == Some(&b'"') {
        input.next();
        field.push(b'"');
    } else {
        in_quotes = false;
        closed_quote = true;
    }
} else {
    field.push(byte);
```

## 00:13:00 — Keep the field and the unresolved decision.

Keep the same source. We have Noah in the record and say in the unfinished note. The first quote of the pair has arrived.

The chunk ends here. Leave the record and field exactly where they are. Only the available input has run out.

We also need one fact: we saw a quote inside quoted contents, and the next byte must tell us what it means.

Give that situation a name: Quote Seen. The name records a question still waiting for an answer. It doesn't mean the field is finished.

Here comes the next chunk. Its first byte is another quote. Now we can resolve the question without rereading the earlier text.

The pair contributes one quote to the field. Switch back to reading quoted contents, ready for h, i, and whatever follows.

We kept exactly the information needed for the next decision. Chunk boundaries can move; the meaning of the bytes stays the same.

Code: [rust/lib.rs](rust/lib.rs), starting at line 141.

```rust
State::QuoteSeen if byte == b'"' => {
    self.field.push(b'"');
    self.state = State::Quoted;
}
```

## 00:14:05 — Four situations, four enum variants.

What other situations must survive between bytes? Before any content in a field, we're at Start. An opening quote is allowed here.

After ordinary unquoted content begins, we're in Bare. Commas and record endings can finish it, but a quote here is an error.

After an opening quote, we're in Quoted. Commas and line breaks are content. A quote takes us to the pending decision.

That pending situation is Quote Seen. Another quote adds one content quote; a legal separator closes the field or record.

These are alternatives, so Rust gives us an enum. The state variable holds one variant at a time, matching our one current situation.

We didn't start by inventing four boxes. Each variant preserves a distinction that changed what the next byte was allowed to do.

Compare an empty bare field with an empty quoted field. Their text can be identical, but their next legal actions differ. Contents alone aren't enough.

Code: [rust/lib.rs](rust/lib.rs), starting at line 10.

```rust
pub enum State {
    Start,
    Bare,
    Quoted,
    QuoteSeen,
}
```

## 00:15:15 — The state belongs with the unfinished work.

Our local variables used to disappear when the parsing function returned. But waiting for another chunk must not erase unfinished work.

Put the state, field bytes, and current record into a Decoder. This struct keeps them together across calls.

Its push method accepts one byte; u eight is Rust's one-byte integer. The mutable borrow lets this call update the decoder's unfinished work.

Most bytes don't complete a record. After an ordinary letter, push returns a successful result containing None: no record ready yet.

At a record boundary, it returns Some with the completed record. The record moves out; the decoder is ready to build the next one.

A malformed byte returns an error instead. Result separates success from failure. Inside success, Option separates a ready record from unfinished work.

This is our core's boundary: one byte in, a state update, and possibly one complete record out. It needs no file handle or application schema.

Code: [rust/lib.rs](rust/lib.rs), starting at line 43.

```rust
pub struct Decoder {
    state: State,
    field: Vec<u8>,
    row: Record,
    offset: usize,
    field_start: usize,
    started: bool,
    after_cr: bool,
    closed: bool,
}

```

## 00:16:28 — Execute the current state, then the byte.

Run the original shopping note through this decoder. After Mira and the separating comma, we're at Start for the second field.

A quote at Start switches to Quoted. It doesn't push a quote into the field. It only changes how following bytes will be read.

In Quoted, bread, the comma, the space, and milk all take the content path. Watch the current line and the growing cell together.

The next quote switches to Quote Seen. It adds no content yet. At final end of input, that quote can legally close the field.

Rust's match selects the arm for the current state. An if guard narrows an arm to a particular next byte, such as another quote.

Once the special quote cases are handled, the common separator branch can finish a field or record. That work doesn't need to be duplicated.

We now have a byte-driven decoder. Feed the same input in one piece or several pieces; keep the decoder, and it keeps the decisions consistent.

Code: [rust/lib.rs](rust/lib.rs), starting at line 133.

```rust
match self.state {
    State::Quoted => {
        if byte == b'"' {
            self.state = State::QuoteSeen;
        } else {
            self.field.push(byte);
        }
    }
    State::QuoteSeen if byte == b'"' => {
```

## 00:17:39 — Only final EOF can finish the input.

Consider this unfinished quoted note. We've read tea, but there is no closing quote. Now the current chunk ends.

Nothing is wrong yet. More bytes may arrive. Keep the Quoted state and the text tea, and wait for the next piece.

Now change just one fact: the input source says this is the final end. No more bytes will arrive. The closing quote is missing.

The finish method reports an unclosed quote. It points to the final byte position, where the missing continuation became certain.

If the state had been Quote Seen instead, a closing quote was already available. Final end could finish that field and its record.

A bare field can also finish at final end. A file does not need a line break after its last record in our chosen rules.

So push means here's a byte, and finish means there will never be another one. Don't use finish to mean please wait for the next chunk.

Code: [rust/lib.rs](rust/lib.rs), starting at line 190.

```rust
    if self.state == State::Quoted {
        return Err(
            self.error(ErrorKind::UnclosedQuote, self.offset)
        );
    }
    if !self.started {
        return Ok(None);
    }
    self.end_field()?;
    Ok(Some(self.end_record()))
}
```

## 00:18:44 — Two bytes can form one record boundary.

Some files end a record with two bytes: carriage return, then line feed. On screen, C R and L F label those otherwise invisible bytes.

Outside quotes, the carriage return finishes the last field, but our rules still require the following line feed before delivering the record.

Put a chunk boundary between them. Keep the completed fields, and remember that we are waiting for line feed. Nothing is delivered twice.

When line feed arrives, clear that pending flag and deliver the record once. Start the next record after the pair.

This flag is separate from the four field states. It records a pending record ending, after field processing has already done its job.

If another byte arrives instead, report an error at that byte. If final end arrives, report the missing line feed at final end.

Inside a quoted field, both bytes are contents. The quoted path keeps them exactly. Our line-ending rule only applies outside quotes.

Code: [rust/lib.rs](rust/lib.rs), starting at line 125.

```rust
if self.after_cr {
    if byte != b'\n' {
        return Err(self.error(ErrorKind::ExpectedLf, at));
    }
    self.after_cr = false;
    return Ok(Some(self.end_record()));
}
self.started = true;
```

## 00:19:56 — A chunk can even split a character.

So far, our examples used mostly one-byte letters. Now import Zoë and café. The accent is part of the value and must survive unchanged.

UTF eight stores some characters in several bytes. Zoom into the final é: these two bytes belong to one character.

A read may stop between them. If we turn each byte into a character immediately, we won't recover the original é correctly.

Instead, keep raw content bytes in the field vector. The first byte waits there. The next chunk adds the remaining byte.

When the field is complete, String from UTF eight validates the whole sequence and gives us the actual text café.

We can still recognize commas and quotes while scanning. A continuation byte in UTF eight cannot be the comma byte or the quote byte.

That gives us a clean boundary: the scan identifies field structure, and string conversion validates the completed text. A byte is not automatically a character.

Code: [rust/lib.rs](rust/lib.rs), starting at line 94.

```rust
fn end_field(&mut self) -> Result<(), CsvError> {
    let bytes = std::mem::take(&mut self.field);
    let text = String::from_utf8(bytes).map_err(|_| {
        self.error(ErrorKind::InvalidUtf8, self.field_start)
    })?;
    self.row.push(text);
    self.state = State::Start;
    self.field_start = self.offset;
    Ok(())
}
```

## 00:21:07 — Tell the caller what failed, and where.

A caller needs more than something went wrong. Here the x after the closing quote violates a specific rule at a specific source position.

Count source bytes from zero as push consumes them. Keep the current position before incrementing, so an error can point to the byte being examined.

Store the error kind and its byte offset together. Rust's error enum names the failure; the error struct carries its location.

Invalid UTF eight is detected when a field completes. Our small implementation points that error to the start of the source field, not a guessed character index.

The question mark after end field means: if this operation failed, return that error now. On success, continue with the next operation.

After failure or final completion, mark the decoder closed. Accidentally feeding it again produces a closed error instead of continuing from damaged partial work.

The application can choose how to display the error. The parser supplies structured facts. It doesn't print a message or terminate the process itself.

Code: [rust/lib.rs](rust/lib.rs), starting at line 28.

```rust
pub struct CsvError {
    /// Zero-based source byte position.
    /// InvalidUtf8 points to the field's start.
    pub offset: usize,
    pub kind: ErrorKind,
}
```

## 00:22:25 — Move the cuts. Keep the answer.

Now test the promise we've been making. This small record has an escaped quote, a multibyte character, and a two-byte line ending.

Feed the entire byte sequence to one decoder. Save the delivered record and any error. That's our reference result.

Then cut after the first byte. Feed the two pieces to a fresh decoder. Compare the result with the reference.

Move the cut one byte at a time through the whole source, including between quotes, inside é, and between carriage return and line feed.

Finally, feed one byte per chunk. There is still only one call to finish, after all the chunks have arrived.

Do the same with malformed inputs. The error kind and source offset must stay the same too. Delivery boundaries shouldn't change the diagnosis.

This is a useful test pattern beyond C S V: vary how input is delivered while keeping its content fixed. The logical result should remain stable.

Code: [tests/streaming.rs](tests/streaming.rs), starting at line 38.

```rust
for cut in 0..=bytes.len() {
    assert_eq!(
        chunks(bytes, &[cut]),
        (expected.clone(), None),
        "cut {cut}, {input:?}"
    );
```

## 00:23:35 — The application asks for one record.

The decoder understands bytes. But an application usually wants a complete record: the next shopper, with all that shopper's fields together.

Let the application ask for the next record. The reader consumes input until the decoder produces one, then hands that record over.

Mira and tea are ready. Return them now. The application can process that shopper before we've parsed every shopper in the file.

Some input may already be buffered after Mira's line. Keep those unread bytes. Asking for one record doesn't mean the operating system supplied exactly one record.

On the next request, resume at the first unread byte. Noah and milk become the next record, using the same decoder instance.

This avoids collecting the entire file inside the reader. It still needs an input buffer and space for the record currently being built.

A single enormous quoted record can still be enormous in memory. Streaming means incremental delivery here; it doesn't make the size of a record disappear.

Code: [checkpoints/06_reader.rs](checkpoints/06_reader.rs), starting at line 8.

```rust
for result in csv::Records::new(&input[..]) {
    let record = result?;
    println!("{record:?}");
}
```

## 00:24:50 — A read fills space, not a CSV record.

What does a read actually give us? We provide a byte buffer. The input source reports how many positions it filled this time.

That count can be smaller than the buffer's capacity. Only the filled prefix is valid input; the unused positions must not be scanned.

Keep two indexes: pos, the next unread position, and len, the end of the filled prefix. Advance pos after taking one byte.

A record can finish before pos reaches len. Return the record and keep both indexes, so the remaining input stays available.

When pos equals len, the filled prefix is exhausted. Read again, reset pos to zero, and set len to the new byte count.

In this blocking reader, a successful read of zero bytes means final end of input. That's when we call the decoder's finish method.

The I O layer decides which bytes are available. The decoder decides where records end. Neither layer should pretend those boundaries are the same.

Code: [rust/lib.rs](rust/lib.rs), starting at line 266.

```rust
Ok(n) => {
    self.pos = 0;
    self.len = n;
}
Err(e)
```

## 00:25:59 — Record, error, or finished.

Rust's Iterator interface matches the way the application is asking: give me the next item, or tell me there are no more items.

Our item is itself a Result. A successful item holds one record. A failed item holds an error encountered while trying to get that record.

So Some of Ok means here's a record. Some of Err means here's a failure. None means the iteration is finished.

That None has a different job from the decoder's successful None. The decoder means this byte hasn't completed a record yet. The iterator must keep working.

Its loop reads or takes another buffered byte, feeds the decoder, and repeats until a record, an error, or final completion is available.

The associated Item type describes what each successful call to next can yield. The next method borrows the reader mutably because it advances that reader.

An empty input gives None immediately after final completion. A blank line gives Some of Ok containing one empty field, then eventually None.

Code: [rust/lib.rs](rust/lib.rs), starting at line 248.

```rust
impl<R: Read> Iterator for Records<R> {
    type Item = Result<Record, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        loop {
            if self.pos == self.len {
```

## 00:27:14 — Follow one request through the loop.

Let's execute one next call. The buffer starts empty, so the reader fills it. The filled prefix happens to contain Mira's whole line and part of Noah's.

Take M, then i, r, and a. Each push succeeds without producing a record. The next loop keeps going.

The comma finishes Mira's field, but the record is incomplete. Still no item to return. Tea fills the second field.

The line feed finishes tea and the record. Push now returns a ready record. Next immediately returns that record to the application.

Notice where pos stopped: at Noah's first unread byte. The buffer and its indexes remain inside the reader after the function returns.

A later next call resumes there. If that prefix runs out mid-field, the loop reads another chunk while the decoder preserves the partial field.

That's the composition: the outer loop waits for a useful result, while the inner decoder handles one byte at a time. Each has a small, explicit job.

Code: [rust/lib.rs](rust/lib.rs), starting at line 283.

```rust
        match self.decoder.push(byte) {
            Ok(Some(row)) => return Some(Ok(row)),
            Ok(None) => {}
            Err(e) => {
                self.done = true;
                return Some(Err(ReadError::Csv(e)));
            }
        }
    }
}
```

## 00:28:27 — Finish once. Report errors once.

What if the last record has no line ending? The loop consumes tea, but it still hasn't received a complete record from push.

The next read returns zero. Now call finish. The decoder completes the last field and gives us the final record.

Mark the iterator done before returning it. A later call to next returns None immediately. It doesn't call finish a second time.

A syntax error follows the same stopping rule: return the error once, then stop. Earlier records already delivered to the application remain valid.

Reading can fail too. An interrupted read is retried. Other read failures become I O errors, distinguished from malformed C S V.

The wrapper enum keeps those two error sources separate. Formatting an error produces readable text; the stored error still carries its original structured information.

The reader never invents an empty record to represent a failure. Data, failure, and final completion each have their own return shape.

Code: [rust/lib.rs](rust/lib.rs), starting at line 258.

```rust
Ok(0) => {
    self.done = true;
    return match self.decoder.finish() {
        Ok(Some(row)) => Some(Ok(row)),
        Ok(None) => None,
        Err(e) => Some(Err(ReadError::Csv(e))),
    };
}
Ok(n) => {
    self.pos = 0;
    self.len = n;
}
Err(e)
```

## 00:29:40 — Change the input source, keep the CSV rules.

We've used a byte slice so the examples stay small. Replace it with an opened file, and the C S V decisions should stay exactly the same.

The reader asks for the Read capability: something that can fill a byte buffer. It doesn't need to know how that source stores its bytes.

In Records of R, R is the chosen source type. The bound R colon Read tells Rust which operation this reader needs from that type.

Our tests supply tiny reads on purpose. The same adapter runs against a file or the test source, and the decoder still sees one byte at a time.

This abstraction pays for itself here: we can change delivery and test failures without changing quotation rules or field building.

It is a blocking adapter. A nonblocking source that temporarily has no data needs a different waiting interface; we don't pretend its temporary state is final end.

The complete source checkpoint includes initialization, error formatting, and the finished loop. The active path we've just followed is the same path in that code.

Code: [rust/lib.rs](rust/lib.rs), starting at line 226.

```rust
pub struct Records<R> {
    input: R,
    decoder: Decoder,
    buffer: [u8; 4096],
    pos: usize,
    len: usize,
    done: bool,
}
```

## 00:30:58 — A field is text. Its meaning belongs to the app.

Suppose these records have a name and a quantity. The parser returns the text one two, which the application still needs to interpret as a quantity.

Is the first row a header? The C S V core doesn't guess. The application can take the first record as names because its own contract says so.

Does every later row need exactly two fields? That's another application rule. Our core can recognize a record even when its field count differs.

The application checks the field count, then converts the quantity string to an integer. A negative quantity might parse as an integer but still violate this application's rules.

Those failures are not quote errors. Keep format errors, number conversion errors, and domain rule failures distinguishable so callers can respond correctly.

The parser's output is the boundary: complete text fields grouped into records. Application code builds meaning from those values after the format is understood.

Change the second column to a shopping note. The C S V reader stays the same. Only the application's interpretation changes.

Code: [checkpoints/07_application.rs](checkpoints/07_application.rs), starting at line 22.

```rust
let record = result?;
if record.len() != 2 {
    return Err("expected two fields".into());
}
let quantity: i32 = record[1].parse()?;
if quantity < 0 {
    return Err("quantity must not be negative".into());
}
orders.push(Order {
```

## 00:32:19 — Predict the finished table.

Time for a fresh example. The first record starts with A. Its note contains quotation marks, an accented character, a comma, and a line break.

Before we run it, look at the quotes and separators. How many fields will the first record have? Take a moment; we'll show the complete answer.

The comma after A finishes field one. An opening quote begins the note. The next two quotes contribute one quote to its contents.

Café follows, including the two-byte é. Another doubled quote adds a closing quote inside the note. The comma and line break still belong to that note.

Next completes the note's second line. The closing quote leaves quoted contents, and the comma finishes that field. There's now a final empty field.

Carriage return plus line feed finishes the first record with three fields. B, an empty middle field, and done form the next three-field record.

Compare the table with the source. Every content character has a destination. Format quotes and separators did their jobs without becoming extra field contents.

## 00:33:39 — Now move only the delivery boundaries.

Keep that exact input and exact expected table. Now split the delivery between a quote pair, inside é, and between carriage return and line feed.

At the split quote, retain Quote Seen. When the next quote arrives, append one quote and resume quoted contents.

At the split character, retain the raw field bytes. Completing the byte sequence later lets the field convert to the same text.

At the split line ending, retain the pending line-feed flag and the completed fields. Deliver the record when line feed arrives.

The finished table is unchanged. What crossed each pause was the small amount of context needed to finish the current decision.

Move the cuts again, or feed one byte at a time. Our tests check that the delivered records and any error position remain identical.

When input delivery changes, keep logical boundaries under the parser's control. Remember unresolved work, rather than letting the transport decide what a value means.

## 00:34:51 — Reuse the principle, not the CSV rules.

Where else does this way of thinking help? Imagine a message stream that gives a length first, followed by that many message bytes.

A read could stop halfway through the length or halfway through the message. Again, the read boundary is not the message boundary.

That parser would remember an unfinished length or how many message bytes remain. It wouldn't need C S V's quoted and bare field states.

The reusable idea is to identify what makes a result complete, then retain the context needed to reach that point across incomplete deliveries.

Give each component a small job with a clear handoff. The source supplies bytes, the format parser supplies values, and the application gives those values meaning.

Then test the boundary by changing delivery without changing content. It's a direct way to catch assumptions that were accidentally hidden in the first happy-path example.

Different formats have different completion rules. Reuse the method of finding and preserving those rules, not a state machine copied from the wrong problem.

## 00:36:08 — A comma gets its meaning from context.

Return to Mira's shopping note one last time. We started with three broken pieces. Now the same input becomes the two intended fields.

We built that result from small decisions: keep content, finish a field, finish a record, and change how punctuation is interpreted when quoting requires it.

The difficult moments told us what to remember. Quote Seen preserves an unresolved quote. Field bytes preserve unfinished text. The record keeps fields that belong together.

The reader carries that work across reads and hands over one completed record at a time. Its buffer boundary never pretends to be a C S V boundary.

The linked source includes the runnable checkpoints, complete reader, format policy, and boundary tests. The transcript and chapters let you return to a specific decision.

To extend this reader, begin with a small input and the result you want. Trace the new decision, then add only the state needed to make it correctly.

That's the finished path from text to useful records. The comma didn't change. We learned to keep the context that tells us what it means.
