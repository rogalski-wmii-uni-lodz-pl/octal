# octal
This repository contains code for an experimental solver for octal games, and results generated by it.
It uses the sparse space phenomenon to speed up the calculation speed for octal games.

This represents an ongoing effort to replicate and extend the work of Achim Flammenkamp, who maintains the only effort known to me to find periods in three digit octal games.

[http://wwwhomes.uni-bielefeld.de/achim/octal.html](http://wwwhomes.uni-bielefeld.de/achim/octal.html)

Values calculated so far:
| game | n | log<sub>2</sub>(n) | period found? | max(G(n)) | 
|---|---|---|---|---|
| [0.014](results/0.014) | [68719476736](results/0.014/68719476736) | 36 | no :x: | 392 |
| [0.034](results/0.034) | [137438953472](results/0.034/137438953472) | 37 | no :x: | 256 |
| [0.161](results/0.161) | [274877906944](results/0.161/274877906944) | 38 | no :x: | 158 |
| [0.167](results/0.167) | [1099511627776](results/0.167/1099511627776) | 40 | no :x: | 64 |
| [0.172](results/0.172) | [68719476736](results/0.172/68719476736) | 36 | no :x: | 387 |

## results
Files in the results subdirectory are frequencies of Sprague-Grundy values of a game.

The results are files organized as follows:
* directory is named after the game,
* frequencies of Sprague-Grundy values of a game are stored in each of the files in the directory,
* the file name is the number of heapsizes calculated (so 68719476736 contains the frequencies after calculating values from 0 to 68719476736),,
* each line in each of the files is in the following format: ```nimber frequency```.
