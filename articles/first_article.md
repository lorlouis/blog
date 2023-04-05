---
Title: This is an article
Author: Louis
Date: 2023-04-03
Blurb: This is a small description of this article
---
# Text is the universal interface

> *This is the Unix philosophy. Write programs that do one thing and do it well.*
>
> *Write programs to work together. Write programs that handle text streams,*
>
> ***because that is a universal interface.***
>
> -- [Douglas McIlroy](https://www.azquotes.com/quote/819507)


If your ever hung in UNIX circles you have probably have seen that quote
being paraphrased around. I know this is a summarisation and I *know* most
people won't care but, **what kind of text streams are we talking about?**
There have been a lot different incompatible character encodings used
throughout the years.

## The early days of ASCII

I think it is a fair assumption to say that most programmers are familiar with
ASCII, some might even know the code point of certain characters by heart 48
for '0', 64 for 'A', 97 for 'a' and so on. Before ASCII came long (1963), computer
manufacturers would roll their own character encoding with no regard to
interoperability between vendors or even machines from the same vendor. IBM
the leading computer vendor at the time had at some point 9 different character
encodings used throughout their different devices[^2].  ASCII was designed to be
used by both computer and teletypes which means that the character set was
required to support both common teletype control codes (BEL, CR, LF, etc) as well
as common characters used in English and characters used for mathematical notation (who could forget ¬). It's also interesting to note that they recognised that 128
characters would not be enough to support other languages so ASCII implemented
an ESC character (decimal 27) to let the machine at the end of the line know that
you were about to send codes that should not be decoded as ASCII. In 1963 the
American National Stander Institute published the first version of ASCII. This
new standard was sure to change the world overnight, no more cursed EBCDIC
*right?* Well in an unsurprising turn of events no one really cared, at least
for another 18 years.

## One? encoding to rule them all

In 1981 IBM released the IBM PC, which made use of ASCII. This microcomputer
took the world by storm. Due to IBM using mostly off the shelf components many
clones came out and the microcomputer landscape suddenly became dominated by
one architecture. This popularised ASCII to the point of becoming the de-facto
character encoding used for computers. The IBM PC was an 8 bit machine as due
to ASCII only covering the bits 0-6 the 7th bit was left unused and the
engineers decided to make use of that 7th bit and add 128 new characters to the
computer's character ROM this extension now called
[code page 437](https://en.wikipedia.org/wiki/Code_page_437).
Many of those code pages were standardised but out of the bunch one became
pretty prevalent. In 1987 ISO codified a certain extensions to ASCII Latin-1.
Latin-1 was designed to cover most languages making use of the Latin alphabet.
Some languages were missing characters notably the French œ and the German ß
but digraphs were commonly used as a substitute. It was still good enough to
defined as the default encoding for any MIME beginning with "text/" in
HTTP 1.1's standard[^3] To recap this leaves us with 3 commonly used version of:w
ASCII.
```
ASCII(7bit) | ASCII+CODE PAGE 437(8bit) | LATIN-1(8bit)
```
Code page 437 being incompatible with LATIN-1 means that if you were to upload
your old "ASCII" that you made on your IBM PC a few years back to the internet
all of the drawing characters would be replaced with a mess of accented
letters.

## On the other side of the globe

You see English is a fairly easy language to fit in a computer it does not use
any characters outside of the common Latin alphabet shared amongst many
European languages. Or at least it does not use any since it got rid of
characters such as thorn 'þ', eth 'ð' and yogh 'ʒ' [due to importing printing
types from Belgium and the
Netherlands](https://arro.anglia.ac.uk/id/eprint/703215/1/25HillFinalDV.pdf)
neither of these countries' languages containing these characters.

But in the 80's there was another country which was making rapid progress in
the world of computers Japan. Japanese uses a very different set of characters
compared to English. It uses Katakana ~ 36, Hiragana ~ 46, and Kanji which are
character imported from china and the set of those that are commonly used
counts about 2000 individual characters.

To represent 2000 individual state you would need 11 bits. 1980's computers
were mostly 8 bit machines which means that in order to fit those states in
a word some sort of multi byte encoding would need to be devised.

TODO talk about shift jis

When talking about early digital character encodings most people know about
[Morse code](https://en.wikipedia.org/wiki/Morse_code) but to be frank in the
context of this article it is not super interesting. I'd like to talk about
[Baudot code](https://en.wikipedia.org/wiki/Baudot_code).

### Baudot code

The Baudot code was invented by
[Émile Baudot](https://en.wikipedia.org/wiki/%C3%89mile_Baudot) in 1870 and is
a 5 bit text encoding scheme capable encoding 32 characters.


#### Diagram of the code and their associated 

```
| Decimal |   Letter  |   Figure  |
| ------- | --------- | --------- |
|    0    |   Blank   |   Blank   |
|    1    |     T     |     5     |
|    2    |    CR     |    CR     |
|    3    |     O     |     9     |
|    4    |   Space   |   Space   |
|    5    |     H     | (nothing) |
|    6    |     N     |    ,      |
|    7    |     M     |    .      |
|    8    | Line Feed | Line Feed |
|    9    |     L     |     )     |
|   10    |     R     |     4     |
|   11    |     G     |     &     |
|   12    |     I     |     8     |
|   13    |     P     |     0     |
|   14    |     C     |     :     |
|   15    |     V     |     ;     |
|   16    |     E     |     3     |
|   17    |     Z     |     "     |
|   18    |     D     |     $     |
|   19    |     B     |     ?     |
|   20    |     S     |    BEL    |
|   21    |     Y     |     6     |
|   22    |     F     |     !     |
|   23    |     X     |     /     |
|   24    |     A     |     -     |
|   25    |     W     |     2     |
|   26    |     J     |     '     |
|   27    | Fig Shift | (nothing) |
|   28    |     U     |     7     |
|   29    |     Q     |     1     |
|   30    |     K     |     (     |
|   31    | Letr Shift| (nothing) |
```
source[^1]

text this is text wow. Checkout <http://example.com> it's a really cool website
I swear.

![this is a weird image](media/idkman.jpg "idk man")

[^1]: <https://cs.stanford.edu/people/eroberts/courses/soco/projects/2008-09/colossus/baudot.html>
[^2]: <http://edition.cnn.com/TECH/computing/9907/06/1963.idg/>
[^3]: <https://datatracker.ietf.org/doc/html/rfc2068#section-3.7.1>
