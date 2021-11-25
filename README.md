# Chess Repertoire Optimizer

This is the Chess Repertoire Optimizer, or short CRO.
It's a tool to help structure your chess opening repertoire to get the most out of your time.

## Motivation

Chess opening preparation is a lot of work.
Repertoires can quickly span thousands of different positions.
But the time we can spend on opening work is limited.

When creating your own repertoire, it is therefore critical to include all
the important lines and to avoid overpreparing minor variations.
However, checking a repertoire manually for lines to add or remove is lot of work.

Wouldn't it be nice if there were a tool to help you build a well-rounded repertoire
with no holes or useless lines?

## Description

CRO checks your opening lines against a chess database, analysing how useful each prepared move is.
It does so by calculating how frequently you will encounter each position from
the move statistics in the database.
The more your opponents play into a certain line,
the more often you will have this position in a game,
and the more useful it is to have this position prepared in your repertoire.

CRO can identify:
* important (i.e. very frequent) lines that you are missing from your repertoire
* unimporant (i.e. very rare) lines in your repertoire that you can remove, reducing your workload

Currently lichess is used to provide the opening database.
This means you can filter the database by time control or rating range -
ensuring the analysis is based on your actual opposition.

## Contribute

I've created this as a tool to help myself in my chess preparation.
Even though I've made it public, it wasn't designed with other users in mind.
The way it works and outputs data is good for me, but probably does not fit a wider audience.
The code is a mess.

But if you find CRO interesting and want to help or have some ideas, you're welcome!
Create an issue, a pull request or whatever. Every input is appreciated.

## License

This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <https://unlicense.org>
