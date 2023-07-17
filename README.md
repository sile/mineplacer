Mineplacer
==========

This is a browser game inspired by Minesweeper.

You are the person preparing for a game of Minesweeper by placing mines!

[Play!](https://sile.github.io/mineplacer/)

How to Play
-----------

- Select a game level (8x15 or 16x30 or 16x30 with wormholes).
- Click on a cell to place a mine there.
- A digit in a cell indicates the number of mines that should be placed around the cell (including the cell itself).
- :warning: mark indicates there are too many mines around the cell.
- When wormholes exist, a part of non-mine cells are hidden.
- The condition for winning is that all mines are placed in the correct positions.

Custom Mode
-----------

The board size and the number of mines/wormholes can be customized by using the following query string parameters:
- `width` (between 16 and 64, default to 16)
- `height` (between 16 and 64, default to 30)
- `mines` (between 1 and 999, default to 99)
- `wormholes` (between 0 and 999, default to 99)

For example, you can play with the maximum settings via the following URL:
- https://sile.github.io/mineplacer?width=64&height=64&mines=999&wormholes=999

![Mineplacer](web/image.jpg)
