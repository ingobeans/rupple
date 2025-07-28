# rupple
![image](https://github.com/user-attachments/assets/30a1da99-10b1-43e4-8e20-cfd77295c460)

rupple is a repl for rust! i made it because [evcxr](https://github.com/evcxr/evcxr), the only other rust repl i know about, is really slow (at least on my machine), requires rust nightly, has a shitload of dependencies, and simply refuses to run on one of my computers.

rupple features:
* run rust code in repl environment with output
* roughly 5x faster than evcxr on my machine
* no nightly required
* crossplatform
* figures out when input is incomplete, i.e. when you open a closure without closing it, so you can submit more lines

install: `cargo install --git https://github.com/ingobeans/rupple.git`

## todo

* add tests (i dont trust that i have covered all edge cases with my custom rust parser)
* syntax highlighting as you type
* ability to move cursor up and down when typing multiple lines