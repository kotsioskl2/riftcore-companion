; RiftCore Companion - NSIS Professional Windows Installer Script
; Ultra-lightweight, solid LZMA compression

!include "MUI2.nsh"
!include "FileFunc.nsh"

!define PRODUCT_NAME "RiftCore Companion"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "kotsioskl2"
!define PRODUCT_WEB_SITE "https://github.com/kotsioskl2/riftcore-companion"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\RiftCore.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKCU"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "..\installer_output\RiftCore-Setup-1.0.0.exe"
InstallDir "$LOCALAPPDATA\Programs\RiftCore"
InstallDirRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails show
ShowUnInstDetails show
RequestExecutionLevel user
SetCompressor /SOLID lzma

; Visual Branding
!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

; Installer Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\RiftCore.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch RiftCore Companion"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Language
!insertmacro MUI_LANGUAGE "English"

Section "MainSection" SEC01
  ; Close existing instance if running
  nsExec::Exec 'taskkill /F /IM RiftCore.exe'

  SetOutPath "$INSTDIR"
  SetOverwrite on

  File "..\RiftCore_Rust_Portable\RiftCore.exe"
  File "..\RiftCore_Rust_Portable\WebView2Loader.dll"

  SetOutPath "$INSTDIR\dist"
  File /r "..\RiftCore_Rust_Portable\dist\*.*"

  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Create Start Menu Shortcuts
  CreateDirectory "$SMPROGRAMS\RiftCore Companion"
  CreateShortCut "$SMPROGRAMS\RiftCore Companion\RiftCore Companion.lnk" "$INSTDIR\RiftCore.exe"
  CreateShortCut "$SMPROGRAMS\RiftCore Companion\Uninstall RiftCore.lnk" "$INSTDIR\uninstall.exe"

  ; Create Desktop Shortcut
  CreateShortCut "$DESKTOP\RiftCore Companion.lnk" "$INSTDIR\RiftCore.exe"

  ; Register in Windows Add/Remove Programs (Installed Apps)
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\RiftCore.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoModify" 1
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoRepair" 1
SectionEnd

Section Uninstall
  ; Terminate process if running during uninstall
  nsExec::Exec 'taskkill /F /IM RiftCore.exe'

  ; Remove Shortcuts
  Delete "$DESKTOP\RiftCore Companion.lnk"
  Delete "$SMPROGRAMS\RiftCore Companion\RiftCore Companion.lnk"
  Delete "$SMPROGRAMS\RiftCore Companion\Uninstall RiftCore.lnk"
  RMDir "$SMPROGRAMS\RiftCore Companion"

  ; Remove Files and Directories
  RMDir /r "$INSTDIR\dist"
  Delete "$INSTDIR\RiftCore.exe"
  Delete "$INSTDIR\WebView2Loader.dll"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  ; Delete Registry Keys
  DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
  DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_DIR_REGKEY}"
SectionEnd
