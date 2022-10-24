#! /bin/bash

# Iterate over all files in the current directory
for file in docs/*.ipynb
do
    # Get the name of the file without the extension
    filename=$(basename "$file")
    # Delete the extension of the filename
    filename="${filename%.*}"
    # Check if the filename contains the string "BlindAI-" or "BlindAI_"
    if [[ $filename == *"BlindAI-"* ]]; then 
        # If it does, then remove the string "BlindAI-" from the filename and the extension
        newfilename=${filename//BlindAI-/}
        # now sed the file to replace the string "About this example" with "Example of BlindAI deployment with newfilename"
        sed -i "s/About this example/Example of BlindAI deployment with $newfilename/g" $file
    elif [[ $filename == *"BlindAI_"* ]]; then
        # If it does, then remove the string "BlindAI-" from the filename
        newfilename=${filename//BlindAI_/}
        # now sed the file to replace the string "About this example" with "Example of BlindAI deployment with newfilename"
        sed -i "s/About this example/Example of BlindAI deployment with $newfilename/g" $file
    fi 
done