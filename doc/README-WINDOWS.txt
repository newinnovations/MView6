Automated Windows build of MView6
---------------------------------

These files are unsigned and Windows will probably warn you before running them.
The warning stems from the fact that they are downloaded from the internet.
The good news is they are compiled fully within the GitHub ecosystem using GitHub
actions, which makes them saver than most downloads. As always use on your own risk.

You can remove the flag using the Unblock-File command before unzipping

   Unblock-File mview6-windows.zip

You can always check the source of a file on Windows using:

   Get-Content -Path .\mview6-windows.zip -Stream Zone.Identifier

   # if you just unblocked the file, you will get an error here as there is
   # no Zone.Identifier anymore
