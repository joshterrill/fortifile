on open fileList
    repeat with aFile in fileList
        set thePath to POSIX path of aFile
        do shell script "/Applications/FortiFile.app/Contents/MacOS/FortiFile " & quoted form of thePath
    end repeat
end open
