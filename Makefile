# ============================================================================
# Makefile - Comandos úteis para desenvolvimento
# ============================================================================

# Variáveis
BINARY_NAME := tcx_image_overlay
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
DEBUG_DIR := $(TARGET_DIR)/debug

# Cores para output
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m # No Color

.PHONY: help build release debug clean test fmt clippy check install run example setup-fonts

# Default target
help: ## Mostra esta mensagem de ajuda
	@echo "$(GREEN)TCX/FIT Image Overlay Tool$(NC)"
	@echo "Comandos disponíveis:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(YELLOW)%-15s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Compila o projeto em modo debug
	@echo "$(GREEN)Compilando em modo debug...$(NC)"
	cargo build

release: ## Compila o projeto em modo release (otimizado)
	@echo "$(GREEN)Compilando em modo release...$(NC)"
	cargo build --release
	@echo "$(GREEN)Binário disponível em: $(RELEASE_DIR)/$(BINARY_NAME)$(NC)"

debug: build ## Alias para build

clean: ## Remove arquivos de compilação
	@echo "$(YELLOW)Limpando arquivos de compilação...$(NC)"
	cargo clean

test: ## Executa todos os testes
	@echo "$(GREEN)Executando testes...$(NC)"
	cargo test

test-verbose: ## Executa testes com output detalhado
	@echo "$(GREEN)Executando testes (verbose)...$(NC)"
	cargo test -- --nocapture

fmt: ## Formata o código usando rustfmt
	@echo "$(GREEN)Formatando código...$(NC)"
	cargo fmt

clippy: ## Executa clippy para análise de código
	@echo "$(GREEN)Executando clippy...$(NC)"
	cargo clippy -- -D warnings

check: fmt clippy test ## Executa formatação, clippy e testes

install: release ## Instala o binário no sistema
	@echo "$(GREEN)Instalando $(BINARY_NAME)...$(NC)"
	cargo install --path .

debug-paths: ## Mostra onde o programa procura pelos arquivos
	@echo "$(GREEN)Caminhos de busca para arquivos:$(NC)"
	@echo "$(YELLOW)Diretório atual:$(NC) $(pwd)"
	@echo "$(YELLOW)Executável:$(NC) ./target/release/tcx_image_overlay"
	@echo "$(YELLOW)Estrutura atual:$(NC)"
	@find . -name "*.png" -o -name "*.ttf" | head -10
	@echo "$(YELLOW)Testando busca de marcas d'água:$(NC)"
	@if [ -f "img/garmin_white.png" ]; then \
		echo "$(GREEN)✅ img/garmin_white.png encontrada$(NC)"; \
	else \
		echo "$(RED)❌ img/garmin_white.png NÃO encontrada$(NC)"; \
	fi
	@if [ -f "img/garmin_black.png" ]; then \
		echo "$(GREEN)✅ img/garmin_black.png encontrada$(NC)"; \
	else \
		echo "$(RED)❌ img/garmin_black.png NÃO encontrada$(NC)"; \
	fi

# Comando de teste com debug extra
run-debug: release ## Executa com informações de debug
	@echo "$(GREEN)Executando com debug de caminhos...$(NC)"
	@echo "$(YELLOW)Diretório atual: $(pwd)$(NC)"
	@echo "$(YELLOW)Listando arquivos em img/:$(NC)"
	@ls -la img/ 2>/dev/null || echo "$(RED)Diretório img/ não existe$(NC)"
	@if [ -f "exemplo.jpg" ] && [ -f "exemplo.tcx" ]; then \
		./target/release/tcx_image_overlay --image-path exemplo.jpg --activity-path exemplo.tcx --output-path debug_result.png; \
	else \
		echo "$(RED)Erro: Arquivos exemplo.jpg e exemplo.tcx necessários para teste$(NC)"; \
	fi
run-example: ## Executa um exemplo (requer arquivos de teste)
	@echo "$(GREEN)Executando exemplo...$(NC)"
	@if [ -f "exemplo.jpg" ] && [ -f "exemplo.tcx" ]; then \
		cargo run -- --image-path exemplo.jpg --activity-path exemplo.tcx; \
	else \
		echo "$(RED)Erro: Arquivos exemplo.jpg e exemplo.tcx necessários$(NC)"; \
		echo "$(YELLOW)Coloque seus arquivos de teste na raiz do projeto$(NC)"; \
	fi

run-fit-example: ## Executa exemplo com arquivo FIT
	@echo "$(GREEN)Executando exemplo FIT...$(NC)"
	@if [ -f "exemplo.jpg" ] && [ -f "exemplo.fit" ]; then \
		cargo run -- --image-path exemplo.jpg --activity-path exemplo.fit --output-path resultado_fit.png; \
	else \
		echo "$(RED)Erro: Arquivos exemplo.jpg e exemplo.fit necessários$(NC)"; \
		echo "$(YELLOW)Coloque seus arquivos de teste na raiz do projeto$(NC)"; \
	fi

# Setup do ambiente
setup-fonts: ## Verifica e cria estrutura de fontes e marcas d'água
	@echo "$(GREEN)Verificando estrutura de arquivos...$(NC)"
	@mkdir -p fonts img
	@echo "$(YELLOW)=== FONTES ===$(NC)"
	@if [ ! -f "fonts/DejaVuSans.ttf" ]; then \
		echo "$(RED)❌ fonts/DejaVuSans.ttf não encontrada$(NC)"; \
		echo "$(YELLOW)   Baixe de: https://dejavu-fonts.github.io/$(NC)"; \
	else \
		echo "$(GREEN)✅ fonts/DejaVuSans.ttf$(NC)"; \
	fi
	@if [ ! -f "fonts/FontAwesome.ttf" ]; then \
		echo "$(RED)❌ fonts/FontAwesome.ttf não encontrada$(NC)"; \
		echo "$(YELLOW)   Baixe de: https://fontawesome.com/$(NC)"; \
	else \
		echo "$(GREEN)✅ fonts/FontAwesome.ttf$(NC)"; \
	fi
	@echo "$(YELLOW)=== MARCAS D'ÁGUA ===$(NC)"
	@if [ ! -f "img/garmin_white.png" ]; then \
		echo "$(RED)❌ img/garmin_white.png não encontrada$(NC)"; \
		echo "$(YELLOW)   Veja setup_watermarks.md para instruções$(NC)"; \
	else \
		echo "$(GREEN)✅ img/garmin_white.png$(NC)"; \
	fi
	@if [ ! -f "img/garmin_black.png" ]; then \
		echo "$(RED)❌ img/garmin_black.png não encontrada$(NC)"; \
		echo "$(YELLOW)   Veja setup_watermarks.md para instruções$(NC)"; \
	else \
		echo "$(GREEN)✅ img/garmin_black.png$(NC)"; \
	fi
	@echo "$(GREEN)Verificação concluída!$(NC)"

check-files: setup-fonts ## Alias para setup-fonts

create-sample-watermarks: ## Cria marcas d'água de exemplo (requer ImageMagick)
	@echo "$(GREEN)Criando marcas d'água de exemplo...$(NC)"
	@mkdir -p img
	@if command -v convert >/dev/null 2>&1; then \
		convert -size 200x50 xc:transparent -font Arial -pointsize 24 -fill white -gravity center -annotate +0+0 "GARMIN" img/garmin_white.png 2>/dev/null || echo "$(YELLOW)Erro ao criar marca d'água branca$(NC)"; \
		convert -size 200x50 xc:transparent -font Arial -pointsize 24 -fill black -gravity center -annotate +0+0 "GARMIN" img/garmin_black.png 2>/dev/null || echo "$(YELLOW)Erro ao criar marca d'água preta$(NC)"; \
		echo "$(GREEN)✅ Marcas d'água de exemplo criadas!$(NC)"; \
	else \
		echo "$(RED)ImageMagick não encontrado. Instale com:$(NC)"; \
		echo "$(YELLOW)  Ubuntu/Debian: sudo apt install imagemagick$(NC)"; \
		echo "$(YELLOW)  macOS: brew install imagemagick$(NC)"; \
		echo "$(YELLOW)  Ou veja setup_watermarks.md para outras opções$(NC)"; \
	fi

# Comandos de desenvolvimento
dev: ## Modo de desenvolvimento com watch
	@echo "$(GREEN)Iniciando modo de desenvolvimento...$(NC)"
	cargo watch -x 'build'

dev-test: ## Modo de desenvolvimento com testes automáticos
	@echo "$(GREEN)Iniciando testes automáticos...$(NC)"
	cargo watch -x 'test'

# Benchmarks e profiling
bench: ## Executa benchmarks (se disponíveis)
	@echo "$(GREEN)Executando benchmarks...$(NC)"
	cargo bench

profile: release ## Cria build com informações de profiling
	@echo "$(GREEN)Compilando com profiling...$(NC)"
	RUSTFLAGS="-g" cargo build --release

# Documentação
docs: ## Gera documentação
	@echo "$(GREEN)Gerando documentação...$(NC)"
	cargo doc --open

docs-private: ## Gera documentação incluindo itens privados
	@echo "$(GREEN)Gerando documentação completa...$(NC)"
	cargo doc --document-private-items --open

# Verificações de segurança
audit: ## Executa auditoria de segurança
	@echo "$(GREEN)Executando auditoria de segurança...$(NC)"
	cargo audit

# Limpeza profunda
deep-clean: clean ## Limpeza profunda incluindo dependências
	@echo "$(YELLOW)Limpeza profunda...$(NC)"
	cargo clean
	rm -rf ~/.cargo/registry/index/*
	rm -rf ~/.cargo/git/checkouts/*

# Informações do sistema
info: ## Mostra informações do ambiente
	@echo "$(GREEN)Informações do ambiente:$(NC)"
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo "Target directory: $(TARGET_DIR)"
	@echo "Binary name: $(BINARY_NAME)"

# Release e distribuição
package: release ## Cria pacote para distribuição
	@echo "$(GREEN)Criando pacote...$(NC)"
	@mkdir -p dist
	@cp $(RELEASE_DIR)/$(BINARY_NAME) dist/
	@cp README.md dist/
	@cp LICENSE dist/
	@echo "$(GREEN)Pacote criado em dist/$(NC)"

# Validação completa antes de commit
pre-commit: clean check ## Validação completa antes de commit
	@echo "$(GREEN)Validação pre-commit concluída!$(NC)"

# Quick start para novos desenvolvedores
quickstart: setup-fonts build ## Setup rápido para novos desenvolvedores
	@echo "$(GREEN)Setup inicial concluído!$(NC)"
	@echo "$(YELLOW)Próximos passos:$(NC)"
	@echo "1. Adicione suas fontes em fonts/"
	@echo "2. Adicione imagens de marca d'água em img/"
	@echo "3. Execute 'make run-example' para testar"