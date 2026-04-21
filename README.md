claudflared:
https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.msi

cloudflared tunnel -url http://localhost:3001

this app requires mkcert to run locally with https. To install mkcert, you can use
choco install:
```
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
```

mkcert:
```
choco install mkcert
```

https://localhost:1420/terminal.html?ui=https://mekou-project.github.io/LookAI/FruitCatch/&app=https://mekou-projects.github.io/FruitCatch/dist/fruitcatch.js
npm install @mekou/engine-api@latest