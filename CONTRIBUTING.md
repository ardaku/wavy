# Contributing
This repository accepts contributions.  Ideas, questions, feature requests and
bug reports can be filed through GitHub issues.

Pull Requests are welcome on GitHub.  By committing pull requests, you accept
that your code might be modified and reformatted to fit the project coding style
in order to improve the implementation.  Contributed code is considered licensed
under the same license as the rest of the project unless explicitly agreed
otherwise.  See the `LICENSE-ZLIB` and `LICENSE-APACHE` files.

Please discuss what you want to see modified before filing a pull request if you
don't want to be doing work that might be rejected.

## git workflow

Please file PR's against the `master` branch (it's the default, so super easy!).

## Adding Support For Other Operating Systems
Async:
 - Prefer to use a port of `smelling_salts` to the target operating system, if
   possible

Sound format:
 - standard sound hardware uses i16(LE), so default to converting to Ch16
 - web uses f64s for sound, so convert to Ch64
 - other targets may have a specific format that makes more sense (guaranteed
   support)
   
### Speaker Configuration
Some platforms may only have support up to stereo, when initializing speaker,
don't try anything higher than the user's requested sample format.
 0. Mono / (Front) Left - **Mono**
 1. (Front) Right - **Stereo**
 2. Back Left
 3. Back Right - **Surround 4**
 4. Front Center
 5. LFE - **Surround 5.1**
 6. Side Left
 7. Side Right - **Surround 7.1**
