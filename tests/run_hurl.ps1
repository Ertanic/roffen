$projectName = "server"

Start-Process cargo "build -p ${projectName}" -Wait -NoNewWindow
$server = Start-Process cargo "run -p ${projectName}" -NoNewWindow -PassThru

hurl --test --variables-file ./tests/hurl/vars.env ./tests/hurl/

Stop-Process $server