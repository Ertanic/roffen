$projectName = "server"

$server = Get-Process -Name $projectName -ErrorAction Ignore
if (!$server) {
    Write-Output "No started server, start one"
    Start-Process cargo "build -p ${projectName}" -Wait -NoNewWindow
    $server = Start-Process cargo "run -p ${projectName}" -PassThru -WindowStyle Hidden
} else {
    Write-Output "Found started server"
}

Start-Sleep -Seconds 2

hurl --test --variables-file ./tests/hurl/vars.env ./tests/hurl/

Write-Output "Stopping server..."
Start-Sleep -Seconds 2
Stop-Process $server