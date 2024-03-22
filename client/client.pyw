import os 
lib = os.path.dirname(os.path.realpath(__file__)) + "\\core.dll"
__import__("mediaplayer").run(lib)
