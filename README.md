# game-of-life
An implementation of a Conway's Game of Life environment on GBA. The ultimate game should have two modes: Edit and Run mode which can be switched between with the Start button (Enter in mGBA). In Edit mode the player can change cells between Alive and Dead. Then in Run mode the player can observe the automata advance according to the rules of Conway's Game of Life.

## Why?
Why not?

## How to Build
Follow the instructions for System Setup here: https://docs.rs/gba/latest/gba/

## How to Play
The game boots in Edit mode and will show a cursor. Press directional keys to move the cursor or A to toggle the current cell state under the cursor.

If you want to completely randomize the screen, press Select while in Edit mode.

To Run Conway's Game of Life, simply press Start and watch it run! If you want to edit an intermediate state, simply press Start again to go back to Edit mode.

Some interesting patterns are available here: https://pi.math.cornell.edu/~lipa/mec/lesson6.html

You can read about the very simple rules that give life to the cellular automata here: https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life#Rules 