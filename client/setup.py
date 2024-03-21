from distutils.core import setup, Extension
import os

packageName = "mediaplayer"

module = Extension(packageName, sources=["loader.c"])
setup(name=PackageName, version="1.0", ext_modules=[module])
