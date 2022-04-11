# Copy the quoteverification.dll to an C:WINDOWS\system32, requires running as administrator
$binReleasePath = ".\third_party\attestationLib\Build\Release\out\bin\Release\"
$quoteLibName = "Quoteverification.dll"
$quoteLibPath = $binReleasePath + $quoteLibName
$quoteLibNewName = "quoteverification.dll"
Rename-Item -Path $quoteLibPath -NewName $quoteLibNewName
$quoteLibPath = $binReleasePath + $quoteLibNewName
Copy-item -Path $quoteLibPath -Destination "C:\WINDOWS\system32" -Force
