---
Title: I wrote a bug and It made me reflect on OOP
Author: Louis
Date: 2024-05-01
Blurb: It's not an OOP bashing post, surprisingly
---
# I wrote a bug and It made me reflect on OOP

## The feature

I wrote a piece of code that would search for matches in a text block and try
to correlate text position with line numbers.

```rust
let re = Regex::new('substr');
// impl Iterator<Item=usize>
let source = re.find(text_block).map(|m| m.start());

let line_ends = [10, 14, 30];

for p in positions {
    let line_idx = line_ends.upper_bound(&p);
    ...
}
```

The implementation was made to be generic and accept any iterator of `usize` as
a source. This made testing easier and I didn't have to specify the whole type
of `re.find(text_block).map(|m| m.start())` which is quite wordy.

## A performance improvement

A few days after writing this patch, I found myself profiling the system, and
this part of the system turned out to be a bit slow. You might have already
noticed, but the regex always returns positions in sorted order: `1, 3, 6,
67...`. This made it pretty easy to ignore parts of the line-end array that had
already been searched.

```rust
let re = Regex::new('substr');
// impl Iterator<Item=usize>
let source = re.find(text_block).map(|m| m.start());

let line_ends = [10, 14, 30];

let mut off = 0;

for p in positions {
    let line_idx = line_ends[off..].upper_bound(&p);
    off = line_idx;
    ...
}
```

## The bug

Requirements changed, along with some code and a new way to search for text was
added. It was still an iterator of `usize` and mostly returned positions in
increasing order, so it appeared to work great. It was faster than the previous
regex method, but it would sometimes return unsorted results sadly none of the
test cases caught that behaviour, so we ended up missing some line indices from
the returned value.

```rust
let source = SuffixFinder::new("ends with this", text_block);

let line_ends = [10, 14, 30];

let mut off = 0;

for p in positions {
    // `p` could sometimes be lower than the position
    // present at `line_ends[off]` because the
    // positions are not returned in sorted order
    let line_idx = line_ends[off..].upper_bound(&p);
    off = line_idx;
    ...
}
```

The fix was pretty simple but I kept thinking about this bug.

## Why is OOP relevant to this discussion

When I was learning programming, a huge part of the curriculum was dedicated to
so-called "object-oriented design". If I were to model the previous problem in
terms of object inheritance and interfaces, it would look like this.

```text
               _____________________________________________
              |              Interface Searcher             |
              | * fn find_line(line_ends: [usize]) -> usize |
              |___~_line_ends.upper_bound(self.p)___________|
 ___________________//________________________     ____\\______
|           Interface SortedSearcher          |   |SuffixFinder|
| * fn last_pos() -> usize                    |
| * fn find_line(line_ends: [usize]) -> usize |
|   ~ line_ends[self.last_pos()..]            |
|___~__________.upper_bound(self.p)___________|
           _____//____
          |RegexFinder|

fn find_position_line(s: Interface Searcher, line_ends: [usize]) -> [usize]
~   let lines = Vec::new();
~   loop
~       let p = s.find_line(line_ends);
~       if p == usize::MAX
~           return lines
~       lines.push(p)
```

The optimisation would be implemented using specialisations and only trigger
when an object inherits from `SortedSearcher`. I would probably have had to
write a newtype that would have wrapped the regex type since it comes from a
library.

Generally, I dislike this type of modelling, it tends to result in poor data
locality due to a lot of objects having to be allocated in languages like Java.
And, in my opinion, it makes the code harder to think about,
`find_position_line` function now depends on two abstract classes implementing
parts of it's logic, so it requires a lot of jumping around in the code and
every time logic is updated in the `Searcher`, `SortedSearcher`'s
implementation needs to be checked and/or updated.

But had I had taken the time to model multiple levels of interfaces and written
the logic inside those interfaces (dependency injection) I would not have
written that bug, and that bugs *me*.
