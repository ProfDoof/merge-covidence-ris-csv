# Merge Covidence Tags into Exported Zotero RIS

I recently wanted to export my completed
review from Covidence into Zotero for the rest 
of the process I will be doing. However, when I
exported the file, I noticed that I lost all
my tags I had added. I was very prolific with my
tags, so I didn't want to redo it manually. 

This is a simple Rust program that imports the
RIS file produced by Covidence for Zotero and
the CSV produced by Covidence for Excel and 
then merges them together. 

It would be ideal if Covidence just threw this
functionality into their export process but
here you go. 

In the future, if I have time, I will add a web
component to this but until then, if you need
this for some reason, you'll need to 
1. install Rust using [rustup](https://rustup.rs/)
2. download this project (using git or just 
downloading a zip folder by clicking the big 
green code button on the page and then 
pressing the Download as Zip button)
3. Unzip the folder if you downloaded it
4. start a terminal (the same one you used to 
   install rustup)
5. navigate to the folder containing the code
   in this project
6. Finally, you will need to run it via the command
line with the command 
    ```
    cargo run -- <your Covidence Zotero RIS file> <your Covidence CSV file> -o <The filename you want to write the merged RIS file to>
    ```

    The filename you write to should probably
    have the extension `.ris` to ensure that it
    works correctly with Zotero.

If you need more help than this, feel free to
make a GitHub issue and I will try to help as
I am able