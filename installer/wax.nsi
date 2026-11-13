; waX VPN installer (NSIS)
!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "x64.nsh"
!include "WinVer.nsh"

!ifndef WAX_VERSION
!define WAX_VERSION "0.1.0"
!endif

Name "waX VPN"
BrandingText "waX VPN - waxvpn.pages.dev"
OutFile "waX-VPN-Setup-${WAX_VERSION}-x64.exe"
InstallDir "$PROGRAMFILES64\waX VPN"
InstallDirRegKey HKLM "Software\waX VPN" "InstallDir"
RequestExecutionLevel admin

; File/product metadata on the installer itself. Unsigned executables with no
; VERSIONINFO are a common antivirus ML false-positive trigger, and the
; v1.0.29 installer was flagged by Windows Defender under exactly that profile.
!searchparse /noerrors "${WAX_VERSION}" "" WAX_VMAJ "." WAX_VMIN "." WAX_VPATCH
!searchreplace WAX_VPATCH_CLEAN "${WAX_VPATCH}" "-preview" ""
VIProductVersion "${WAX_VMAJ}.${WAX_VMIN}.${WAX_VPATCH_CLEAN}.0"
VIAddVersionKey "ProductName" "waX VPN"
VIAddVersionKey "CompanyName" "vio137"
VIAddVersionKey "FileDescription" "waX VPN Setup"
VIAddVersionKey "FileVersion" "${WAX_VERSION}"
VIAddVersionKey "ProductVersion" "${WAX_VERSION}"
VIAddVersionKey "LegalCopyright" "MIT License"
VIAddVersionKey "OriginalFilename" "waX-VPN-Setup-${WAX_VERSION}-x64.exe"

!define MUI_ABORTWARNING
!define MUI_ICON "payload\wax.ico"
!define MUI_UNICON "payload\wax.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP "payload\installer-wizard.bmp"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "payload\installer-header.bmp"
!define MUI_FINISHPAGE_RUN "$INSTDIR\waX VPN.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch waX VPN"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "payload\LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Function .onInit
    ${IfNot} ${RunningX64}
        MessageBox MB_ICONSTOP "waX VPN requires 64-bit Windows."
        Abort
    ${EndIf}
    ${IfNot} ${AtLeastWin10}
        MessageBox MB_ICONSTOP "waX VPN requires Windows 10 or later (64-bit)."
        Abort
    ${EndIf}
    SetRegView 64
FunctionEnd

Function un.onInit
    SetRegView 64
FunctionEnd

Section "waX VPN" SecMain
    ; Older previews used the default 32-bit registry view. Remove only
    ; this product's stale registration before writing the x64 entry.
    SetRegView 32
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN"
    DeleteRegKey HKLM "Software\waX VPN"
    SetRegView 64
    SetOutPath "$INSTDIR"
    SetShellVarContext all

    ; The new bundled helper works even when upgrading an old CLI. No taskkill.
    InitPluginsDir
    SetOutPath "$PLUGINSDIR"
    File /oname=wax-installer.exe "payload\wax.exe"
    File /oname=wireguard.dll "payload\wireguard.dll"
    nsExec::Exec '"$PLUGINSDIR\wax-installer.exe" installer-stop "$INSTDIR"'
    Pop $0
    ${If} $0 != 0
        IfSilent +2
        MessageBox MB_ICONSTOP "Could not stop the installed waX VPN app. Close it, then run the installer again."
        SetErrorLevel 1
        Abort
    ${EndIf}
    IfFileExists "$INSTDIR\wax.exe" 0 +3
        nsExec::Exec '"$PLUGINSDIR\wax-installer.exe" disconnect'
        Pop $0
    SetOutPath "$INSTDIR"

    File "payload\waX VPN.exe"
    File "payload\wax.exe"
    File "payload\wireguard.dll"
    File "payload\LICENSE.txt"
    File "payload\LICENSE-astryx.txt"
    File /oname=LICENSE-egui-software.txt "..\vendor\egui-software\LICENSE-MIT"
    File "payload\README.txt"
    File "payload\LICENSE-wireguard-nt.txt"

    WriteRegStr HKLM "Software\waX VPN" "InstallDir" "$INSTDIR"

    WriteUninstaller "$INSTDIR\uninstall.exe"

    CreateDirectory "$SMPROGRAMS\waX VPN"
    CreateShortcut "$SMPROGRAMS\waX VPN\waX VPN.lnk" "$INSTDIR\waX VPN.exe"
    CreateShortcut "$SMPROGRAMS\waX VPN\Uninstall waX VPN.lnk" "$INSTDIR\uninstall.exe"
    CreateShortcut "$DESKTOP\waX VPN.lnk" "$INSTDIR\waX VPN.exe"
    CreateShortcut "$SMPROGRAMS\waX VPN\Accelerated graphics.lnk" "$INSTDIR\waX VPN.exe" "--accelerated-graphics"
    CreateShortcut "$SMPROGRAMS\waX VPN\Safe graphics.lnk" "$INSTDIR\waX VPN.exe" "--safe-graphics"

    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "DisplayName" "waX VPN"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "DisplayVersion" "${WAX_VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "Publisher" "vio137"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "URLInfoAbout" "https://waxvpn.pages.dev/"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "UninstallString" '$\"$INSTDIR\uninstall.exe$\"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "QuietUninstallString" '$\"$INSTDIR\uninstall.exe$\" /S'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "InstallLocation" "$INSTDIR"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "NoRepair" 1
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN" "EstimatedSize" $0
SectionEnd

Section "Uninstall"
    SetShellVarContext all
    ; This installation's CLI uses exact-path native process cleanup.
    nsExec::Exec '"$INSTDIR\wax.exe" installer-stop "$INSTDIR"'
    Pop $0
    ${If} $0 != 0
        IfSilent +2
        MessageBox MB_ICONSTOP "Could not stop the installed waX VPN app. Close it, then run uninstall again."
        SetErrorLevel 1
        Abort
    ${EndIf}
    nsExec::Exec '"$INSTDIR\wax.exe" disconnect'
    Pop $0
    Sleep 1000

    Delete "$INSTDIR\waX VPN.exe"
    Delete "$INSTDIR\wax.exe"
    Delete "$INSTDIR\wireguard.dll"
    Delete "$INSTDIR\LICENSE.txt"
    Delete "$INSTDIR\LICENSE-astryx.txt"
    Delete "$INSTDIR\LICENSE-egui-software.txt"
    Delete "$INSTDIR\README.txt"
    Delete "$INSTDIR\LICENSE-wireguard-nt.txt"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"

    Delete "$SMPROGRAMS\waX VPN\waX VPN.lnk"
    Delete "$SMPROGRAMS\waX VPN\Uninstall waX VPN.lnk"
    Delete "$SMPROGRAMS\waX VPN\Accelerated graphics.lnk"
    Delete "$SMPROGRAMS\waX VPN\Safe graphics.lnk"
    RMDir "$SMPROGRAMS\waX VPN"
    Delete "$DESKTOP\waX VPN.lnk"

    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\waX VPN"
    DeleteRegKey HKLM "Software\waX VPN"
    ; User profiles/settings under %APPDATA%\wax-vpn are intentionally kept.
    SectionEnd
