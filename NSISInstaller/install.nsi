; Plugins
!include "MUI2.nsh"


; Installer Content
Name "FFNodes"
!define MUI_ICON "icon.ico"
OutFile "FFNodes Installer.exe"
Unicode True

; Variables
Var StartMenuFolder

; Registry data
InstallDir "$PROGRAMFILES\LFInteractive\FFNodes"
InstallDirRegKey HKCU "Software\LFInteractive\FFNodes" ""
RequestExecutionLevel admin

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_LICENSE "tos.txt"

!insertmacro MUI_PAGE_DIRECTORY

!define MUI_STARTMENUPAGE_REGISTRY_ROOT "HKCU" 
!define MUI_STARTMENUPAGE_REGISTRY_KEY "Software\LFInteractive\FFNodes" 
!define MUI_STARTMENUPAGE_REGISTRY_VALUENAME "Start Menu Folder"
!insertmacro MUI_PAGE_STARTMENU Application $StartMenuFolder

!insertmacro MUI_PAGE_INSTFILES

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "FFNodes" SecFFNodes

  SetOutPath "$INSTDIR\FFNodes"
  File /r "..\FFNodes.Client\bin\Release\net7.0-windows10.0.19041.0\win10-x64\"

  ExecWait '"$INSTDIR\FFNodes.zip" -o "$INSTDIR"' $0

  WriteRegStr HKCU "Software\LFInteractive\FFNodes" "" $INSTDIR
  AccessControl::GrantOnFile "$INSTDIR" "(BU)" "FullAccess"

  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application


  ;Create shortcuts
  CreateDirectory "$SMPROGRAMS\$StartMenuFolder"
  CreateShortcut "$SMPROGRAMS\$StartMenuFolder\FFNodes.lnk" "$INSTDIR\FFNodes\FFNodes.exe"
  CreateShortcut "$INSTDIR\FFNodes.lnk" "$INSTDIR\FFNodes\FFNodes.exe"
  !insertmacro MUI_STARTMENU_WRITE_END

  ; Add custom URL protocol handler
  WriteRegStr HKCR "ffn" "" "URL: FFNodes Protocol"
  WriteRegStr HKCR "ffn\shell\open\command" "" '"$INSTDIR\FFNodes\FFNodes.exe" "%1"'
  
SectionEnd
