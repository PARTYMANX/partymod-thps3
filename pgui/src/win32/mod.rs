pub mod app;
mod button;
mod window;
mod font;

/*
    writing down thoughts about design:

    Essentially we need to make sure updates to state can make their way into UI objects that aren't "listening" to those updates
    How do we do that?

    We have to hash state on each update. so we kind of need to collect view state on each update. kind of goofy.
    What we could do is give each component a function that it uses to update the view. Each view function's result
    could be hashed/cached to determine when we should actually change it...

    Okay yeah that's more or less how react does it. Just provide a hook to adjust variables.
*/
