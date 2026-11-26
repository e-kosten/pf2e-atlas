!define ATLAS_INSTALLER_SUPPORT "${__FILEDIR__}"

; HKCU registration is shared by every install destination for this user.
; CREATE_NEW never touches an existing file. Windows drops the exclusive
; delete-on-close handle on every exit, including a failed or killed setup.
!macro AtlasReserveInstallation
  System::Call 'kernel32::CreateFileW(w "$LOCALAPPDATA\pf2e-atlas-desktop-installer.lock", i 0x10000, i 0, p 0, i 1, i 0x04000000, p 0) p.r0'
  ${If} $0 = -1
    DetailPrint "Could not reserve the Atlas installation. Wait for any other Atlas setup or uninstall to finish, then retry."
    MessageBox MB_OK|MB_ICONSTOP "Could not reserve the Atlas installation. Wait for any other Atlas setup or uninstall to finish, then retry. If this continues, check access to $LOCALAPPDATA\pf2e-atlas-desktop-installer.lock." /SD IDOK
    SetErrorLevel 1618
    Quit
  ${EndIf}
!macroend

!macro AtlasPayload Arguments
  SetOverwrite on
  File "/oname=$PLUGINSDIR\install-atlas-payload.ps1" "${ATLAS_INSTALLER_SUPPORT}\install-atlas-payload.ps1"
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\install-atlas-payload.ps1" ${Arguments} -InstallDirectory "$INSTDIR"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    DetailPrint "$1"
    MessageBox MB_OK|MB_ICONSTOP "Atlas could not finish the installation change.$\r$\n$1" /SD IDOK
    SetErrorLevel 1
    Quit
  ${EndIf}
!macroend
