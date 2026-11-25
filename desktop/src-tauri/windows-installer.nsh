!define ATLAS_INSTALLER_SUPPORT "${__FILEDIR__}"

!macro AtlasCommitPayload
  File "/oname=$PLUGINSDIR\install-atlas-payload.ps1" "${ATLAS_INSTALLER_SUPPORT}\install-atlas-payload.ps1"
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\install-atlas-payload.ps1" -Payload "$PLUGINSDIR\atlas-payload" -InstallDirectory "$INSTDIR"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    DetailPrint "$1"
    MessageBox MB_OK|MB_ICONSTOP "Atlas could not replace the installed files.$\r$\n$1" /SD IDOK
    SetErrorLevel 1
    Quit
  ${EndIf}
!macroend
