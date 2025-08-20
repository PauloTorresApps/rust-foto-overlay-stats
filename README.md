# TCX/FIT Image Overlay Tool

Uma ferramenta em Rust para adicionar overlays de estatísticas de treino a partir de arquivos TCX ou FIT em imagens.

## 🚀 Funcionalidades

- ✅ Suporte para arquivos **TCX** e **FIT**
- ✅ Overlay com estatísticas de treino (tempo, calorias, frequência cardíaca, etc.)
- ✅ Detecção automática de dispositivos Garmin com marca d'água
- ✅ Análise automática de luminosidade para escolha da marca d'água
- ✅ Interface de linha de comando intuitiva
- ✅ Arquitetura modular e extensível

## 📋 Pré-requisitos

- Rust 1.70 ou superior
- Fontes necessárias:
  - `fonts/DejaVuSans.ttf` - Fonte principal para texto
  - `fonts/FontAwesome.ttf` - Fonte para ícones
- Imagens de marca d'água (opcional):
  - `img/garmin_white.png` - Marca d'água branca
  - `img/garmin_black.png` - Marca d'água preta

## 🛠️ Instalação

### Clone o repositório
```bash
git clone https://github.com/usuario/tcx_image_overlay.git
cd tcx_image_overlay
```

### Compile o projeto
```bash
cargo build --release
```

## 📖 Uso

### Sintaxe básica
```bash
./target/release/tcx_image_overlay -i <IMAGEM> -f <ARQUIVO_TCX_OU_FIT> [-o SAIDA]
```

### 📁 Sistema de Saída Automática

O sistema agora organiza automaticamente suas imagens processadas em:

```
~/stats_overlay/
├── 2024-08-20/
│   ├── corrida-matinal-stats-overlay.jpg
│   └── treino-bike-stats-overlay.jpg
├── 2024-08-21/
│   ├── caminhada-stats-overlay.jpg
│   └── natacao-stats-overlay.jpg
└── ...
```

**Estrutura:**
- **Diretório base**: `~/stats_overlay/` (criado automaticamente)
- **Subdiretório por data**: `YYYY-MM-DD/` (data atual)
- **Nome do arquivo**: `nome-original-stats-overlay.extensão`

### Exemplos

#### Saída automática (recomendado)
```bash
./target/release/tcx_image_overlay -i corrida.jpg -f atividade.tcx
# Salva em: ~/stats_overlay/2024-08-20/corrida-stats-overlay.jpg
```

#### Com arquivo FIT
```bash
./target/release/tcx_image_overlay -i bike-treino.png -f garmin.fit
# Salva em: ~/stats_overlay/2024-08-20/bike-treino-stats-overlay.png
```

#### Saída personalizada (opcional)
```bash
./target/release/tcx_image_overlay -i foto.jpg -f treino.tcx -o minha-pasta/resultado.jpg
# Salva em: minha-pasta/resultado.jpg
```

#### Usando nomes longos (também funciona)
```bash
./target/release/tcx_image_overlay \
  --image foto.jpg \
  --file atividade.tcx \
  --output resultado_personalizado.png
```

#### Ajuda
```bash
./target/release/tcx_image_overlay --help
```

### 🎯 Sintaxe Super Simplificada

A nova interface é muito mais fácil de usar:

| Antigo (verboso) | Novo (simples) |
|------------------|----------------|
| `--image-path`   | `-i`           |
| `--activity-path`| `-f`           |
| `--output-path`  | `-o` (opcional)|

**Comando mais simples possível:**
```bash
./target/release/tcx_image_overlay -i foto.jpg -f treino.tcx
```

**✨ O sistema cuida de tudo:**
- ✅ **Cria diretórios automaticamente**
- ✅ **Organiza por data**
- ✅ **Nomeia arquivos inteligentemente**
- ✅ **Evita conflitos de nomes**

## 📁 Estrutura do Projeto

```
src/
├── main.rs              # Ponto de entrada
├── cli.rs               # Interface de linha de comando
├── error.rs             # Sistema de erros
├── constants.rs         # Constantes da aplicação
├── image_processor.rs   # Processamento de imagens
└── parsers/
    ├── mod.rs          # Módulo principal dos parsers
    ├── tcx.rs          # Parser para TCX
    └── fit.rs          # Parser para FIT
```

## 🎨 Funcionalidades do Overlay

O overlay inclui as seguintes informações:

- ⏱️ **Tempo total** - Duração da atividade
- 🔥 **Calorias** - Energia gasta durante o treino  
- ❤️ **Frequência Cardíaca** - Média e máxima
- 📅 **Data** - Data da atividade
- 📱 **Dispositivo** - Nome do dispositivo usado

### Cores das estatísticas
- **Tempo**: Azul (#3498db)
- **Calorias**: Laranja (#e67e22)
- **Frequência Cardíaca**: Vermelho (#e74c3c)
- **Data**: Verde (#2ecc71)
- **Dispositivo**: Cinza (#95a5a6)

## 🏗️ Arquitetura

O projeto segue os princípios de **Clean Architecture** com separação clara de responsabilidades:

### Módulos principais

- **CLI**: Interface de linha de comando com `clap`
- **Parsers**: Módulos especializados para TCX e FIT
- **ImageProcessor**: Lógica de processamento de imagens
- **Error**: Sistema centralizado de tratamento de erros
- **Constants**: Configurações e constantes

### Benefícios da arquitetura

- ✅ **Testabilidade**: Cada módulo pode ser testado isoladamente
- ✅ **Extensibilidade**: Fácil adicionar novos formatos ou funcionalidades
- ✅ **Manutenibilidade**: Código organizado e bem estruturado
- ✅ **Reutilização**: Componentes podem ser reutilizados

## 🧪 Testes

```bash
# Executar todos os testes
cargo test

# Executar testes com output detalhado
cargo test -- --nocapture

# Executar testes específicos
cargo test parsers::tcx
```

## 🔧 Desenvolvimento

### Adicionar novo formato de arquivo

1. Crie um novo módulo em `src/parsers/`
2. Implemente a função de parsing retornando `ActivityData`
3. Adicione o formato em `image_processor.rs`

### Personalizar overlay

- Modifique as constantes em `src/constants.rs`
- Ajuste as cores, ícones ou layout conforme necessário

## 📝 Formatos Suportados

### Arquivos de entrada
- **TCX** (Training Center XML) - Garmin, Polar, etc.
- **FIT** (Flexible and Interoperable Data Transfer) - Garmin, Wahoo, etc.

### Imagens suportadas
- PNG, JPEG, WebP, TIFF, BMP
- Qualquer formato suportado pela crate `image`

## 🤝 Contribuição

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## 📄 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## 🐛 Problemas Conhecidos

- Alguns arquivos FIT muito antigos podem não ter todos os campos
- Fontes devem estar nos caminhos especificados

## 📞 Suporte

Se você encontrar problemas ou tiver sugestões:

1. Verifique os [issues existentes](https://github.com/usuario/tcx_image_overlay/issues)
2. Crie um novo issue com detalhes do problema
3. Inclua exemplos de arquivos problemáticos (se possível)

---

Feito com ❤️ em Rust