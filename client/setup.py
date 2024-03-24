from setuptools import setup, Extension

packageName = "mediaplayer"

ext_module = Extension(name=packageName, sources=["loader.c"])
setup(name=packageName, version="0.1", ext_modules=[ext_module])

print("Done")