All this year is going to be firstly made in rust, I will do performance too. For that I'ma make a general helper that compiles each rust file into an `.so` file with a common exported main function that takes the input byte buffer and returns a solution (hoping all of them will be integers).

I will make an external rust program that takes the day to run and the input, it will buffer and measure the execution speed.
