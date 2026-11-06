# ClipVault 🔒📋

**ClipVault** é um aplicativo desktop moderno, compacto e *dark-first* para Ubuntu GNOME com X11, desenvolvido com **Tauri 2**, **React**, **TypeScript**, **Tailwind CSS** e **Rust**.

Projetado para gerenciar de forma 100% offline o histórico da área de transferência e capturas de tela com consumo mínimo de recursos (CPU e RAM).

---

## ✨ Funcionalidades Principais

- **Monitoramento X11 Nativo via XFixes (0% CPU Ociosa):** Não utiliza polling contínuo. Escuta eventos nativos `XFixesSelectionNotify` no socket X11, acordando o processo apenas quando há novas cópias.
- **Área de Transferência Convencional (`CLIPBOARD`):** Ignora estritamente a seleção primária do mouse (`XA_PRIMARY`), capturando apenas cópias intencionais (`Ctrl+C` / "Copiar").
- **Suporte Completo a Texto e Imagens:** Monitora tanto textos (UTF-8) quanto imagens (`image/png`, screenshots do sistema), gerando miniaturas e previews detalhados.
- **Captura de Tela Integrada:**
  - Tela Inteira (multi-monitores).
  - Janela Ativa (`_NET_ACTIVE_WINDOW`).
  - Região Selecionada com interface interativa.
- **Deduplicação Inteligente e Prevenção de Loops:** Evita gravar itens repetidos consecutivamente e impede loops ao restaurar itens do histórico no clipboard.
- **Bandeja do Sistema (System Tray):** Minimiza para a bandeja sem encerrar o processo. Fecha a janela mantendo o monitoramento em segundo plano.
- **Atalho Global Configurável:** Padrão `Super+Shift+V` para alternar rapidamente a visibilidade da janela.
- **Inicialização Automática:** Integração com FreeDesktop autostart (`~/.config/autostart/clipvault.desktop`).
- **Pausa e Limpeza Imediata:** Botão para pausar o monitoramento a qualquer momento e botão de wipe imediato com exclusão de arquivos em disco.

---

## 🗄️ Arquitetura de Armazenamento (Sem Banco de Dados)

O ClipVault não utiliza SQLite, Postgres ou qualquer banco de dados externo:
- **Metadados:** Armazenados em formato JSON em `~/.cache/clipvault/history.json`.
- **Configurações:** Salvas em `~/.cache/clipvault/settings.json`.
- **Imagens:** Salvas em formato PNG em `~/.cache/clipvault/images/<uuid>.png`.
- **Escrita Atômica:** Todas as gravações utilizam arquivos temporários (`.tmp.<uuid>`) com sincronização em disco (`fsync`) e substituição atômica (`fs::rename`), garantindo que falhas de energia ou travamentos nunca corrompam o histórico.
- **Expiração Automática (24 horas):** Itens com mais de 24 horas são purgados automaticamente na inicialização e a cada 5 minutos em segundo plano. Imagens expiradas são apagadas do disco. Itens fixados (*Favoritos*) são preservados.
- **Limpeza de Órfãos e Limite de Armazenamento:** Varredura periódica para deletar imagens que não constem mais no histórico ativo e controle de limite máximo de itens e espaço em disco (MB).

---

## 🛡️ Segurança e Privacidade

- **100% Offline:** Nenhuma requisição HTTP/HTTPS de telemetria ou rede externa é realizada. O aplicativo só se comunica localmente com o servidor X11.
- **Proteção Heurística contra Conteúdo Sensível:**
  - Detecta janelas ativas de gerenciadores de senhas conhecidos (KeePassXC, 1Password, Bitwarden, KeePass, Seahorse, etc.) via `WM_CLASS` e `_NET_WM_NAME`.
  - Detecta chaves privadas PEM (`BEGIN RSA/OPENSSH PRIVATE KEY`).
  - Valida padrões de cartões de crédito utilizando o algoritmo de Luhn.
  - Detecta tokens de alta entropia (GitHub PATs, Slack tokens, JWTs).
  - Conteúdos identificados recebem flag e opção de ocultação visual imediata no histórico.

---

## 📦 Instalação do Pacote Debian (.deb)

O pacote instalável gerado para o Ubuntu 24.04 LTS (x86_64) está disponível na raiz do repositório:

```bash
# Instalar via dpkg ou apt
sudo dpkg -i clipvault_1.0.0_amd64.deb

# Caso falte alguma dependência do sistema:
sudo apt-get install -f -y
```

Após instalado, o ClipVault aparecerá no menu de aplicativos do GNOME e pode ser executado pelo comando:
```bash
clipvault
```

---

## 🛠️ Desenvolvimento e Testes

### Pré-requisitos
- Ubuntu 24.04 (X11 Session)
- Node.js 22 + pnpm
- Rust stable (rustc, cargo)
- Bibliotecas GTK e WebKit: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`

### Executar Testes Unitários
```bash
cd src-tauri
cargo test
```
*Testes inclusos: deduplicação, prevenção de loops, detecção de conteúdo sensível (Luhn, chaves privadas, gerenciadores de senhas), escrita atômica e limpeza de arquivos órfãos.*

### Compilar Pacote Debian via Docker
Para compilar de forma reprodutiva sem necessidade de instalar dependências de desenvolvimento no host:

```bash
# 1. Construir imagem do builder
docker build -t clipvault-builder:noble -f Dockerfile.builder .

# 2. Gerar o pacote Debian
docker run --rm \
  -e COREPACK_ENABLE_DOWNLOAD_PROMPT=0 \
  -v $(pwd):/workspace \
  -v ~/.cargo/registry:/usr/local/cargo/registry \
  -v ~/.cargo/git:/usr/local/cargo/git \
  -w /workspace \
  clipvault-builder:noble \
  npx --yes @tauri-apps/cli build --bundles deb
```

---

## ⌨️ Atalhos

| Ação | Atalho Padrão |
| :--- | :--- |
| Abrir / Ocultar Histórico | `Super+Shift+V` |
| Focar na Pesquisa | `/` |
| Limpar Pesquisa / Fechar Foco | `Esc` |
| Copiar Item | `Clique no Card` ou `Ícone de Cópia` |
| Fixar Item (Não expira) | `Ícone de Estrela` |
