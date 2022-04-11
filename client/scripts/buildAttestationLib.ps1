# run third_party/attestationLib/release.ps1 to build the attestation library
$libraryBuildScript = ".\third_party\attestationLib\release.ps1"
&$libraryBuildScript

# Copy the libraries to blindai/lib
if (Test-Path '.\blindai\lib') {
    Remove-Item '.\blindai\lib' -Recurse -Force 
}
New-Item -Path '.\blindai\lib' -ItemType Directory
Copy-Item -Path ".\third_party\attestationLib\Build\Release\out\bin\Release\QuoteVerification.dll" -Destination ".\blindai\lib"
Copy-Item -Path ".\third_party\attestationLib\Build\Release\out\lib\Release\*" -Destination ".\blindai\lib\"