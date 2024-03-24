import os

target = os.path.dirname(os.path.abspath(__file__)) + "\\target\\release\\client.dll"

module = __import__("mediaplayer")
output = module.run(target)

# print(target)