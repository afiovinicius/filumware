# Wireless Scanner --- PRD & Technical Specification

**Versão:** 1.0\
**Data:** 11/11/2026\
**Status:** Especificação inicial para implementação\
**Linguagem:** Rust\
**Plataforma-alvo:** Linux (prioridade: Arch Linux)\
**Interface:** TUI com Ratatui + Crossterm\
**Aquisição Wi-Fi:** Linux nl80211 / Generic Netlink\
**Modo operacional:** Monitor (com restauração obrigatória do estado
original)

------------------------------------------------------------------------

## 1. Visão do produto

Wireless Scanner é uma aplicação de terminal (TUI) escrita em Rust para
descoberta, inspeção e monitoramento **passivo** de redes Wi-Fi.

Na primeira versão, a aplicação deve:

1.  Identificar os adaptadores Wi-Fi disponíveis, seus drivers,
    capacidades, interfaces e modos suportados.
2.  Registrar o estado original da interface escolhida.
3.  Preparar a interface em modo monitor, quando suportado e necessário.
4.  Fazer varredura de pontos de acesso e atualizar os resultados
    continuamente.
5.  Exibir ESSID, BSSID, canal, frequência, RSSI em dBm, fabricante
    (OUI) e informações WPS anunciadas.
6.  Permitir navegação, seleção, ordenação por cabeçalho e consulta
    detalhada de uma rede.
7.  Salvar os dados da rede selecionada em JSON.
8.  Encerrar com segurança e restaurar a configuração original da
    interface e do terminal.

A arquitetura deve permitir uma segunda etapa de monitoramento passivo
mais aprofundado, incluindo estações observadas, quadros de
gerenciamento e indicadores de presença de tráfego EAPOL ou elementos
relacionados a PMKID.

### 1.1 Referência visual e funcional

A captura fornecida do `airodump-ng` serve como referência para a futura
tela de monitoramento. Ela apresenta duas áreas conceituais:

-   **Access points:** BSSID, PWR, RXQ, Beacons, #Data, taxa de dados,
    canal, velocidade, criptografia, cipher, autenticação e ESSID.
-   **Stations:** BSSID associado, MAC da estação, PWR, taxa, frames
    perdidos, frames observados, notas e probes.

O produto não deve copiar indiscriminadamente todos os campos como se
fossem equivalentes ou universalmente disponíveis. Cada campo precisa
ter definição, fonte e nível de confiança documentados.

------------------------------------------------------------------------

## 2. Limites funcionais e segurança

O programa deve operar como ferramenta de inventário e diagnóstico
passivo em redes próprias ou cujo monitoramento tenha sido autorizado.

### Incluído

-   Recepção passiva de quadros 802.11 no canal sintonizado.
-   Decodificação de beacons, probe responses e quadros de gerenciamento
    necessários à descoberta.
-   Leitura dos Information Elements de segurança e WPS anunciados.
-   Observação de endereços MAC de estações em quadros recebidos.
-   Correlação observacional entre estação e BSSID, quando houver
    evidência suficiente.
-   Contagem de quadros EAPOL observados e indicação de presença de
    tráfego EAPOL.
-   Indicação de que um elemento/atributo relacionado a PMKID foi
    observado, somente quando o parser conseguir identificá-lo com
    confiança.
-   Exportação de metadados e estatísticas, sem material criptográfico
    reutilizável.

### Fora do escopo

-   Desautenticação, deauth, disassociation ou injeção de quadros.
-   Tentativas de PIN WPS, força bruta, Pixie Dust ou exploração de WPS.
-   Solicitação forçada de handshake ou de PMKID.
-   Extração, armazenamento ou exportação de material de autenticação
    destinado a cracking.
-   Dicionários, cracking offline, recuperação de senha ou teste de
    credenciais.
-   Interferência, jamming ou qualquer tentativa de degradar redes.
-   Promessa de identificar todos os clientes conectados.

**Regra de apresentação:** o aplicativo deve mostrar `observed`,
`not observed` ou `unknown`, e não declarar que uma rede é vulnerável
apenas porque um tipo de quadro foi observado. A presença de EAPOL ou de
um indicador PMKID não comprova, por si só, vulnerabilidade ou
possibilidade de recuperar uma senha.

------------------------------------------------------------------------

## 3. Objetivos e não objetivos

### 3.1 Objetivos da v1

-   Inicialização guiada por etapas, com loading visível na TUI.
-   Descoberta de hardware e capacidades.
-   Uso de modo monitor para captura passiva, quando suportado.
-   Scanner contínuo de access points.
-   Parsing de ESSID, BSSID, canal, frequência, RSSI, segurança, WPS e
    OUI.
-   Tabela navegável e ordenável.
-   Painel de detalhes de uma rede.
-   Cache em memória e exportação JSON.
-   Restauração da interface e do terminal ao encerrar.

### 3.2 Objetivos de arquitetura para a v2

-   Monitoramento passivo de estações.
-   Correlação temporal entre BSSID e estações observadas.
-   Métricas de frames, beacons e perda estimada.
-   Histórico de RSSI por access point.
-   Contagem de quadros EAPOL observados, sem armazenar payloads de
    autenticação.
-   Indicador observacional de elemento PMKID, sem extração/exportação
    de material de autenticação.
-   Visão dividida de access points e estações, inspirada na organização
    visual do airodump-ng.

### 3.3 Não objetivos

-   Substituir NetworkManager ou outro gerenciador de rede.
-   Ser um scanner de vulnerabilidades ofensivo.
-   Garantir descoberta de todos os dispositivos presentes em uma WLAN.
-   Garantir captura de todos os quadros em todos os canais
    simultaneamente com um único rádio.

------------------------------------------------------------------------

## 4. Requisitos funcionais

### RF-001 --- Inicialização e loading

Ao iniciar, mostrar o header da aplicação e uma área central de
progresso. Exibir etapas distintas:

1.  Inicializando terminal.
2.  Descobrindo interfaces wireless.
3.  Identificando driver e chipset, quando disponível.
4.  Consultando capacidades e bandas.
5.  Lendo o estado original da interface.
6.  Preparando modo monitor.
7.  Inicializando captura passiva.
8.  Iniciando descoberta de access points.

A UI não deve congelar durante operações de hardware ou Netlink. Cada
etapa deve publicar eventos de progresso para a camada de apresentação.

### RF-002 --- Descoberta de adaptadores

Identificar:

-   Nome da interface (ex.: `wlan0`).
-   PHY associado (ex.: `phy0`).
-   MAC da interface.
-   Driver e versão, quando disponíveis.
-   Chipset/modelo, quando disponível.
-   Estado administrativo (UP/DOWN).
-   Tipo atual da interface.
-   Bandas e frequências suportadas.
-   Suporte a modo monitor.
-   Limitações relevantes reportadas pelo driver.

Não inventar chipset ou driver quando a fonte do sistema não fornecer
essa informação. Usar `unknown` e registrar a origem.

### RF-003 --- Estado original e restauração

Antes de qualquer alteração, armazenar:

-   Interface e PHY.
-   Tipo original.
-   Estado UP/DOWN.
-   MAC original.
-   Canal/frequência original, quando aplicável.
-   Informações disponíveis sobre o gerenciador de rede.
-   Interfaces virtuais que já existiam no PHY.

Ao finalizar:

1.  Parar a captura.
2.  Fechar sockets e recursos Netlink/PCAP.
3.  Remover somente interfaces virtuais criadas pelo próprio programa.
4.  Restaurar tipo e parâmetros alterados.
5.  Restaurar estado UP/DOWN.
6.  Liberar recursos.
7.  Restaurar o terminal.

Não remover nem reconfigurar interfaces que não tenham sido criadas ou
alteradas pelo aplicativo. Não presumir que mudar de monitor para
managed restaura automaticamente a conexão anterior. Quando houver
NetworkManager, detectar o estado e coordenar a recuperação sem
desabilitar o serviço globalmente.

Implementar limpeza em encerramento normal e sinais tratáveis (`SIGINT`,
`SIGTERM`). Documentar que `SIGKILL`, kernel panic ou perda de energia
impedem garantias de restauração.

### RF-004 --- Modo monitor

O modo escolhido para a aplicação é **monitor**.

-   Verificar suporte do driver e do PHY antes da alteração.
-   Preferir interface monitor dedicada, quando o hardware/driver
    suportar interface virtual concorrente.
-   Caso seja necessário converter a interface existente, guardar o
    estado original e avisar o usuário.
-   Não presumir que uma interface monitor consegue transmitir ou
    receber em todas as bandas/canais.
-   Não executar injeção de quadros.
-   Exibir claramente o canal/frequência atualmente sintonizado.

O scanner deve continuar funcionando em modo monitor por meio de
recepção de beacons e probe responses. Não depender de uma varredura
ativa que exija transmissão de probes.

### RF-005 --- Descoberta de access points

Campos mínimos:


|Campo | Tipo | Definição |
| --- | --- | --- |
| ESSID                   | string opcional         | Nome anunciado; vazio/oculto quando não divulgado |
| BSSID                   | MAC                     | Endereço do access point/BSS |
| Channel                 | inteiro opcional        | Canal derivado da frequência e da banda |
| Frequency               | MHz                     | Frequência observada |
| Signal                  | dBm opcional           | RSSI reportado pelo driver no radiotap ou pela fonte disponível |
| Vendor                  | string opcional         | Fabricante estimado por OUI |
| Security                | lista                   | Protocolos/ciphers/authentication identificados |
| WPS                     | enum/estrutura          | Presença e atributos anunciados |
| First seen              | timestamp               | Primeira observação na sessão |
| Last seen               | timestamp               | Observação mais recente |
| Beacons                |  contador                | Beacons recebidos nesta sessão, quando capturados |
| Data frames             | contador opcional       | Contagem de quadros de dados observados; não é necessariamente tráfego total |


A aplicação deve atualizar os dados sem duplicar redes. A chave primária
de um BSS é o BSSID; ESSID não é chave única.

### RF-006 --- Critério WPS

O resultado booleano solicitado deve seguir estritamente:

`WPS_YES = WPS_PRESENT && PIN_ADVERTISED`

Tabela:

| WPS presente   | PIN anunciado   | PBC anunciado   | Resultado |
| -------------- | --------------- | --------------- | ----------- |
| Não            | Não |             Não |             NO |
| Sim            | Não |             Sim |             NO |
| Sim            | Sim |             Não |             YES |
| Sim            | Sim |             Sim |             YES |
| Sim            | Não |             Não |             NO |

O estado configurado e o PBC não alteram o critério acima.

**Importante:** `YES` significa apenas "WPS presente e método PIN
anunciado". Não significa "PIN explorável", "vulnerável" ou "ataque
possível".

O parser deve analisar atributos WPS binários, não procurar strings na
saída de comandos. Atributos relevantes incluem:

-   Version (`0x104A`)
-   WPS State (`0x1044`)
-   Config Methods (`0x1008`)
-   Device Password ID (`0x1012`)
-   Selected Registrar (`0x1041`)
-   AP Setup Locked (`0x1057`)

A interpretação deve respeitar os valores e bitmasks definidos pela
especificação WPS. Se os atributos estiverem ausentes, truncados ou
inconsistentes, o resultado deve ser `UNKNOWN`, não `NO`.

### RF-007 --- Vendor/OUI

-   Extrair os primeiros 24 bits do MAC quando aplicável.
-   Consultar uma base OUI local, versionada.
-   Não fazer requisição de rede para cada MAC.
-   Permitir fabricante desconhecido.
-   Distinguir fabricante estimado por OUI de fabricante declarado pelo
    próprio dispositivo.
-   Documentar que MAC randomization e OUI localmente administrado podem
    impedir identificação confiável.

### RF-008 --- Tabela e ordenação

Tabela principal:

`ESSID | BSSID | CHANNEL | dBm | WPS | VENDOR`

Comportamento:

-   Atualização em tempo real.
-   Navegação por setas UP/DOWN.
-   Scroll vertical.
-   ENTER abre detalhes da rede.
-   Clique no cabeçalho ordena a coluna.
-   Segundo clique inverte a direção.
-   A seleção permanece associada ao BSSID mesmo após atualização ou
    ordenação.

Ordenação:

| Coluna   | Primeiro clique                 | Segundo clique |
| -------- | ------------------------------- | ------------------------- |
| ESSID    | A → Z                           | Z → A |
| dBm      | sinal mais forte → mais fraco   | mais fraco → mais forte |
| WPS      | YES primeiro                    | NO primeiro |

Para dBm, sinal mais forte significa valor numericamente maior:
`-40 dBm` é mais forte que `-80 dBm`. Valores ausentes devem ficar ao
final, em qualquer direção.

Para WPS, valores desconhecidos devem ficar separados de YES e NO.

### RF-009 --- Detalhes da rede

Ao selecionar um access point, abrir painel com:

-   ESSID e BSSID.
-   Vendor e origem da identificação.
-   Frequência e canal.
-   RSSI atual e histórico disponível.
-   Segurança, cipher e autenticação.
-   WPS presente, PIN anunciado, PBC anunciado, estado configurado e
    bloqueio, quando conhecidos.
-   First seen e last seen.
-   Contadores de beacons/frames, quando disponíveis.
-   Qualidade/confiança de cada campo.

A seleção deve permitir exportar os dados daquela rede em JSON.

### RF-010 --- Estações observadas (preparação para v2)

A versão futura poderá exibir uma segunda tabela, inspirada na seção
`STATION` do airodump-ng:

| Campo       | Definição |
| ----------- | ----------------------------------------------------------- |
| BSSID       | AP ao qual a estação aparenta estar relacionada |
| STATION     | MAC de origem/destino observado |
| PWR         | Sinal reportado para o quadro recebido, quando disponível |
| Rate        | Taxa indicada no quadro/metadado, se disponível |
| Frames      | Quantidade de quadros observados |
| Last seen   | Última observação |
| Probes      | SSIDs em probe requests, se coletados e autorizados |

A correlação entre estação e BSSID precisa ser baseada em evidências de
associação ou tráfego observado. Um quadro isolado não comprova que o
dispositivo esteja atualmente conectado.

O aplicativo deve usar o estado `observed`, não afirmar "connected",
salvo quando houver uma fonte autoritativa que confirme a associação.

### RF-011 --- Indicadores EAPOL e PMKID (v2)

A versão futura pode apresentar indicadores passivos:

-   `EAPOL observed: YES/NO/UNKNOWN`
-   `EAPOL frames observed: count`
-   `PMKID-related element observed: YES/NO/UNKNOWN`

Esses indicadores devem ser calculados sem armazenar, exportar ou
apresentar payloads de autenticação reutilizáveis. Não implementar
captura forçada, deauth, extração de PMKID, exportação de handshakes ou
cracking.

`NO` significa "não observado durante esta sessão/canal/período", e não
"não existe". A cobertura depende do canal, tempo de observação, perdas
de captura e comportamento dos clientes.

------------------------------------------------------------------------

## 5. Requisitos de interface (TUI)

### 5.1 Identidade visual

A interface deve seguir o padrão visual do FILUMWARE apresentado pelo
usuário, sem reutilizar a identidade funcional ou o nome de outro
projeto:

-   Header superior com nome e versão.
-   Box central grande para conteúdo.
-   Footer inferior com atalhos.
-   Bordas discretas.
-   Tipografia e alinhamento consistentes.
-   Paleta escura, com destaque ciano/azul.
-   Estados de loading, erro, vazio e captura ativa.

### 5.2 Layout principal

``` text
┌────────────────────────────────────────────────────────────────────┐
│                         WIRELESS SCANNER                            │
│                              v1.0.0                                  │
├────────────────────────────────────────────────────────────────────┤
│ ACCESS POINTS                                      LIVE / MONITOR   │
│                                                                    │
│ ESSID       BSSID              CHANNEL   dBm     WPS    VENDOR      │
│ HOME_5G     AA:BB:CC:DD:EE:01  36        -42     YES    Vendor A    │
│ OFFICE      AA:BB:CC:DD:EE:02  11        -58     NO     Vendor B    │
│ GUEST       AA:BB:CC:DD:EE:03  6         -73     NO     Unknown     │
│                                                                    │
│                                                                    │
│ Adapter: wlan0 | Driver: ... | PHY: phy0 | Channel: 36             │
├────────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate [ENTER] Details [H] Help [ESC] Menu [R] Refresh [Q] Quit │
└────────────────────────────────────────────────────────────────────┘
```

Os dados acima são fictícios e servem somente para ilustrar o layout.

### 5.3 Popups

**Help (`H`)**

-   Ocupa 90% da largura e 90% da altura disponíveis.
-   Lista atalhos e descrições.
-   Possui rolagem se o conteúdo exceder a tela.
-   ESC fecha/retorna.
-   Q encerra a aplicação.

**Menu (`ESC`)**

Opções:

1.  Voltar à tela anterior.
2.  Central de ajuda.
4.  Encerrar programa.

**Transparência:** Ratatui pode desenhar fundo cinza e sobrepor widgets,
mas transparência real de 70% depende do terminal e não é portável.
Implementar fallback em cinza sólido, sem prometer alpha blending
universal.

### 5.4 Atalhos

| Tecla                | Ação |
| -------------------- | -------------------------------------- | 
| UP/DOWN              | Navegar |
| ENTER                | Selecionar/abrir detalhes |
| H                    | Central de ajuda |
| ESC                  | Menu ou voltar |
| Q                    | Encerrar globalmente |
| R                    | Solicitar nova atualização/varredura |
| PGUP/PGDN            | Rolar uma página |
| Mouse no cabeçalho   | Ordenar coluna |
| Mouse em uma linha   | Selecionar rede |

O Q deve ser reconhecido em todas as telas e popups.

------------------------------------------------------------------------

## 6. Arquitetura e estrutura de diretórios

A estrutura definida pelo produto é:

``` text
src/
├── main.rs
├── core/
│   ├── mod.rs
│   ├── variables.rs
│   └── pipeline.rs
└── utils/
    ├── mod.rs
    ├── ui.rs
    ├── hardware.rs
    ├── wireless.rs
    ├── parsing.rs
    ├── wps.rs
    ├── vendor.rs
    ├── sorting.rs
    ├── archives.rs
    └── terminal.rs
```

### 6.1 Responsabilidades


| Módulo                             |  Responsabilidade |
| ----------------------------------- | ----------------------------------- |
| `main.rs`                           | Inicializar e orquestrar o  pipeline; não conter lógica de domínio |
| `core/variables.rs`                 | Structs, enums, estados e modelos compartilhados |
| `core/pipeline.rs`                  | Coordenar inicialização, descoberta, captura, atualização e encerramento |
| `utils/ui.rs`                       | Header, tabela, footer, popups e desenho da TUI |
| `utils/hardware.rs`                 | Descoberta de interfaces, driver, PHY, chipset e capacidades |
| `utils/wireless.rs`                 | Comunicação nl80211 e gerenciamento da interface |
| `utils/parsing.rs`                  | Parsing de frames e Information Elements |
| `utils/wps.rs`                      | Interpretação dos atributos WPS |
| `utils/vendor.rs`                   | Resolução OUI |
| `utils/sorting.rs`                  | Ordenação e estabilidade da seleção |
| `utils/archives.rs`                 | Cache e exportação JSON |
| `utils/terminal.rs`                 | Raw mode, alternate screen, mouse capture e restauração |


### 6.2 Regra de dependência

-   `main.rs` conhece e chama `core::pipeline`.
-   `core::pipeline` coordena os módulos de `utils`.
-   `utils` contém funções e componentes utilitários.
-   `utils` pode usar tipos de `core::variables`, mas não importa
    `core::pipeline`.
-   `utils` não deve orquestrar o fluxo da aplicação.
-   Evitar estado global mutável; preferir estado explícito passado por
    referência ou canais de eventos.
-   A UI recebe snapshots/eventos e não executa operações de hardware
    diretamente.

### 6.3 Fluxo de eventos

Separar aquisição da renderização:

``` text
nl80211 / capture
       |
       v
Acquisition Worker
       |
       v
Raw Observation
       |
       v
Parser / Normalizer
       |
       v
Network Snapshot
       |
       +------> Cache / JSON
       |
       v
UI Event Channel
       |
       v
Ratatui Renderer
```

A UI não deve bloquear enquanto a aquisição trabalha. Usar canais
assíncronos ou um mecanismo equivalente para enviar atualizações à TUI.

------------------------------------------------------------------------

## 7. Estratégia de aquisição Linux

### 7.1 Interface principal

Usar **nl80211 sobre Generic Netlink** para descoberta e gerenciamento
de interfaces e para operações de wireless suportadas pelo driver.

A captura passiva em modo monitor pode utilizar uma biblioteca Rust de
captura PCAP/libpcap com suporte a radiotap, desde que a dependência e a
configuração sejam documentadas. A aplicação deve manter o processamento
de quadros estritamente passivo.

### 7.2 Separação das fontes

-   **nl80211:** interfaces, PHY, tipos, capacidades, frequências e
    informações de BSS disponibilizadas pelo kernel.
-   **Radiotap/PCAP:** metadados de recepção e quadros 802.11 recebidos
    em monitor mode.
-   **Sysfs:** informações complementares de dispositivo/driver.
-   **Base OUI local:** fabricante estimado.

Não usar parsing de saída de `iw`, `nmcli`, `wash` ou `airodump-ng` como
mecanismo principal. Ferramentas externas podem ser usadas manualmente
para comparação em testes, nunca como dependência funcional obrigatória.

### 7.3 Canal e cobertura

Uma interface monitor normalmente sintoniza um canal por vez. O produto
deve implementar uma política explícita:

-   **Fixed channel:** usuário ou configuração define um canal.
-   **Channel hopping:** alternância entre canais suportados, com
    intervalo configurável e sem injeção.
-   Exibir sempre o canal atual.
-   Associar cada observação ao canal e horário em que foi recebida.
-   Informar que channel hopping reduz a cobertura temporal de cada
    canal.
-   Não somar métricas de períodos/canais diferentes sem preservar essa
    origem.

A primeira entrega pode usar canal fixo ou uma política simples de
hopping passivo, desde que o comportamento seja transparente na UI.

------------------------------------------------------------------------

## 8. Modelo de dados

Exemplo conceitual de tipos:

``` rust
pub struct HardwareInfo {
    pub interface: String,
    pub phy: Option<String>,
    pub driver: Option<String>,
    pub chipset: Option<String>,
    pub mac_address: Option<String>,
    pub original_mode: InterfaceMode,
    pub current_mode: InterfaceMode,
    pub was_up: bool,
    pub supported_bands: Vec<Band>,
    pub monitor_supported: bool,
}

pub enum InterfaceMode {
    Managed,
    Monitor,
    Other(String),
    Unknown,
}

pub struct Network {
    pub essid: Option<String>,
    pub bssid: String,
    pub channel: Option<u16>,
    pub frequency_mhz: Option<u32>,
    pub signal_dbm: Option<f32>,
    pub vendor: Option<String>,
    pub security: SecurityInfo,
    pub wps: WpsInfo,
    pub first_seen: u64,
    pub last_seen: u64,
    pub beacon_count: u64,
}

pub struct WpsInfo {
    pub present: Option<bool>,
    pub pin_advertised: Option<bool>,
    pub pbc_advertised: Option<bool>,
    pub configured: Option<bool>,
    pub setup_locked: Option<bool>,
    pub accepted_by_product_rule: Option<bool>,
}

pub struct StationObservation {
    pub mac: String,
    pub related_bssid: Option<String>,
    pub signal_dbm: Option<f32>,
    pub frames_observed: u64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub association_evidence: AssociationEvidence,
}

pub enum AssociationEvidence {
    ExplicitAssociationFrame,
    ObservedTraffic,
    Unknown,
}
```

Os modelos finais devem derivar `Serialize`/`Deserialize` onde fizer
sentido. Para dados desconhecidos, usar `Option` ou enums explícitos em
vez de inventar valores padrão.

------------------------------------------------------------------------

## 9. Persistência e exportação

Estrutura:

``` text
data/
├── sessions/
│   └── session_<timestamp>.json
├── networks/
│   ├── <BSSID_NORMALIZED>/
│   |   ├── <BSSID_NORMALIZED>.pcapng
│   |   ├── <BSSID_NORMALIZED>.csv
│   |   ├── <BSSID_NORMALIZED>.log
│   |   └── <BSSID_NORMALIZED>.json
│   └── ...
└── oui/
    └── oui.csv
```

### Requisitos

-   Cache em memória durante a sessão.
-   Exportação da rede selecionada.
-   Exportação opcional do snapshot completo da sessão.
-   Escrita atômica: gravar arquivo temporário e renomear ao concluir.
-   Criar diretórios automaticamente.
-   Não salvar payloads de autenticação.
-   Incluir timestamp, interface, modo e canal da observação.
-   Incluir versão do schema para permitir migrações.

Exemplo de JSON:

``` json
{
  "schema_version": 1,
  "captured_at": "2026-11-11T13:43:00-03:00",
  "interface": "wlan0mon",
  "mode": "monitor",
  "network": {
    "essid": "example-network",
    "bssid": "AA:BB:CC:DD:EE:FF",
    "channel": 11,
    "frequency_mhz": 2462,
    "signal_dbm": -67,
    "vendor": "Example Vendor",
    "wps": {
      "present": true,
      "pin_advertised": false,
      "pbc_advertised": true,
      "accepted_by_product_rule": false
    }
  }
}
```

Dados fictícios para ilustrar o formato.

------------------------------------------------------------------------

## 10. Stack e dependências sugeridas

As versões devem ser fixadas e verificadas durante a implementação,
evitando assumir que versões antigas continuam sendo as mais adequadas.

| Componente         | Opção |
| ------------------ | -------------------------------------------------------- |
| Linguagem          | Rust stable |
| TUI                | `ratatui` |
| Terminal/eventos   | `crossterm` |
| Assincronismo      | `tokio` |
| Serialização       | `serde`, `serde_json` |
| Erros              | `thiserror`, `anyhow` |
| Netlink            | crate Rust mantida e compatível com Generic Netlink |
| Captura passiva    | `pcap`/libpcap ou alternativa Rust compatível |
| Parsing binário    | parser próprio com slices e validação de limites |
| Datas              | `chrono` ou `time` |
| Testes             | `cargo test`, fixtures binárias e testes de integração |

Evitar adicionar dependências sem necessidade. O parser deve validar
comprimento, alinhamento, endianness e truncamento dos elementos antes
de acessar campos.

------------------------------------------------------------------------

## 11. Tratamento de erros

A aplicação deve apresentar erros compreensíveis e preservar a
restauração do sistema.

Casos obrigatórios:

-   Nenhuma interface wireless encontrada.
-   Driver não suporta monitor mode.
-   Permissão insuficiente.
-   PHY ocupado ou combinação de interfaces não suportada.
-   Falha ao criar interface monitor.
-   Falha ao abrir socket Netlink/PCAP.
-   Interface removida durante a execução.
-   Canal não suportado.
-   Nenhum access point observado.
-   Frames inválidos ou truncados.
-   Base OUI ausente.
-   Falha de escrita JSON.
-   Terminal redimensionado.
-   Interrupção por sinal.

Não mostrar stack trace bruto como mensagem principal da TUI. Registrar
contexto técnico em log opcional.

------------------------------------------------------------------------

## 12. Testes e critérios de aceitação

### Hardware e inicialização

-   [ ] Detecta todas as interfaces wireless disponíveis.
-   [ ] Registra interface, PHY, driver e capacidades quando
    disponíveis.
-   [ ] Detecta suporte a monitor mode.
-   [ ] Guarda o estado original antes de qualquer alteração.
-   [ ] Não altera interfaces não selecionadas.
-   [ ] Mostra loading sem congelar a TUI.

### Scanner

-   [ ] Descobre BSSID e ESSID anunciado.
-   [ ] Descobre frequência e canal.
-   [ ] Exibe RSSI em dBm, quando fornecido.
-   [ ] Atualiza redes sem criar duplicatas por BSSID.
-   [ ] Preserva timestamps e origem das observações.
-   [ ] Não apresenta valores ausentes como zero.

### WPS

-   [ ] Interpreta atributos WPS binários.
-   [ ] `YES` somente quando WPS está presente e PIN está anunciado.
-   [ ] PBC isolado resulta em `NO`.
-   [ ] Atributos ausentes/inválidos resultam em `UNKNOWN` quando não
    for possível concluir.
-   [ ] Não declara vulnerabilidade com base apenas no anúncio de PIN.

### TUI

-   [ ] Header, painel central e footer.
-   [ ] Navegação UP/DOWN.
-   [ ] Scroll.
-   [ ] ENTER abre detalhes.
-   [ ] Clique no cabeçalho ordena ESSID, dBm e WPS.
-   [ ] Ordenação alterna a direção.
-   [ ] Seleção permanece no mesmo BSSID após refresh/sort.
-   [ ] H abre ajuda.
-   [ ] ESC abre menu/volta.
-   [ ] Q encerra em qualquer tela.
-   [ ] Redimensionamento não quebra o layout.

### Persistência e encerramento

-   [ ] Exporta rede selecionada em JSON.
-   [ ] JSON inclui schema version e timestamps.
-   [ ] Falha de exportação é comunicada sem encerrar abruptamente.
-   [ ] Terminal volta ao modo normal.
-   [ ] Interface volta ao estado original em encerramento normal.
-   [ ] Recursos criados pelo programa são removidos sem tocar em
    recursos preexistentes.

### Testes de parser

-   [ ] Fixtures de beacon com SSID visível.
-   [ ] Fixtures de SSID oculto.
-   [ ] Fixtures WPA2 e WPA3.
-   [ ] Fixtures WPS com PIN.
-   [ ] Fixtures WPS apenas PBC.
-   [ ] Fixtures sem WPS.
-   [ ] Fixtures truncadas e malformadas.
-   [ ] Testes de frequência para canal.
-   [ ] Testes de RSSI ausente.

------------------------------------------------------------------------

## 13. Roadmap

### Fase 1 --- Fundação

-   Estrutura do projeto.
-   Modelos globais.
-   Inicialização/restauração do terminal.
-   Descoberta de hardware.
-   Identificação de driver e capacidades.
-   Gerenciamento seguro do modo monitor.

**Entrega:** aplicação abre, identifica a placa e mostra
loading/diagnóstico.

### Fase 2 --- Scanner

-   Aquisição passiva.
-   Parsing de beacons e probe responses.
-   ESSID, BSSID, canal, frequência e RSSI.
-   Segurança e WPS.
-   Vendor/OUI.
-   Deduplicação e atualização em tempo real.

**Entrega:** tabela de access points funcional.

### Fase 3 --- TUI e persistência

-   Ratatui completo.
-   Navegação e scroll.
-   Ordenação por mouse/cabeçalho.
-   Help e menu.
-   Detalhes da rede.
-   JSON e cache.

**Entrega:** v1 utilizável.

### Fase 4 --- Monitoramento passivo de estações

-   Parser de quadros de gerenciamento.
-   Rastreamento temporal de estações observadas.
-   Correlação de evidências com BSSID.
-   RSSI e contagem de frames.
-   Visão dividida APs/Stations.

**Entrega:** painel de observação passiva inspirado na organização do
airodump-ng.

### Fase 5 --- Indicadores de autenticação

-   Contagem de EAPOL observados.
-   Indicador de presença de elemento relacionado a PMKID, quando
    identificável.
-   Estados `observed`, `not observed`, `unknown`.
-   Relatórios sem material criptográfico reutilizável.

**Entrega:** diagnóstico passivo e auditável, quando entra em minitoramento de uma rede gera arquivos auditável, colleta dados para uso gerando .csv, .pcapng (otimizado com dados de PMKID, EAPOL, tudo compativel com hashcat), esses arquivos são criados e alimentados na fase de monnitoramento exportados para dentro da pasta da rede selecionada.

------------------------------------------------------------------------

## 14. Observações técnicas sobre a referência airodump-ng

A imagem de referência mostra valores como:

-   `PWR`: indicador de potência recebido pela ferramenta; sua
    interpretação depende do adaptador/driver e não deve ser tratado
    como uma escala universal.
-   `RXQ`: estimativa de qualidade de recepção/cobertura de frames,
    conforme implementação.
-   `Beacons`: quantidade de beacons recebidos.
-   `#Data`: quadros de dados observados, não necessariamente bytes ou
    volume total.
-   `CH`: canal.
-   `MB`: taxa/indicador de velocidade anunciado ou observado, sujeito à
    implementação.
-   `ENC`, `CIPHER`, `AUTH`: classificação de segurança.
-   `STATION`: endereço MAC de uma estação observada.
-   `Frames`: quantidade de quadros vistos daquela estação.
-   `Probes`: nomes de rede anunciados em probe requests, quando
    presentes.

Não presumir que `PWR` seja sempre comparável entre adaptadores
diferentes. O campo deve ser exibido como RSSI em dBm somente quando a
fonte fornecer uma medida nessa unidade.

### EAPOL

EAPOL é tráfego relacionado a autenticação 802.1X/EAP e também participa
do processo de autenticação WPA-Personal. A aplicação pode contar
quadros EAPOL observados e indicar atividade durante a janela de
captura. Isso não prova que um handshake completo foi observado.

### PMKID

PMKID é um identificador relacionado ao contexto de chave WPA. A
presença de um campo/elemento relacionado a PMKID deve ser registrada
apenas quando a estrutura correspondente for identificada corretamente.
Não confundir anúncio de capacidade, ocorrência de elemento e captura de
material de autenticação.

### Número de dispositivos

Uma captura passiva não oferece necessariamente a lista completa de
clientes atualmente conectados. Dispositivos silenciosos, economia de
energia, roaming, MAC randomization, canal errado e perdas de recepção
podem ocultar estações. A UI deve dizer **estações observadas**, com
janela temporal, e não "clientes conectados" como certeza, usar o métodos
usados por ferramentas como airodump-ng para mostrar os clientes conectados.

------------------------------------------------------------------------

## 15. Instruções de implementação para o agente de código

1.  Leia este documento integralmente antes de modificar arquivos.
2.  Implemente por fases; não tente construir todas as funcionalidades
    em um único arquivo.
3.  Preserve a estrutura `core/` e `utils/`.
4.  Não coloque lógica de domínio em `main.rs`.
5.  Não faça a UI executar comandos ou operações de hardware.
6.  Não use `iw`, `nmcli`, `wash` ou `airodump-ng` como dependências
    funcionais.
7.  Use nl80211/Generic Netlink para comunicação com o kernel.
8.  Use monitor mode para recepção passiva, verificando suporte do
    adaptador.
9.  Não implemente transmissão de quadros, deauth, tentativas WPS ou
    extração de material de autenticação.
10. Escreva testes unitários para parsing, WPS, frequência, ordenação e
    serialização.
11. Trate campos indisponíveis como desconhecidos, nunca invente
    valores.
12. Garanta que o encerramento restaure o terminal e, em encerramento
    normal, a configuração original da interface.
13. Ao terminar cada fase, apresente arquivos criados, decisões
    técnicas, testes executados e limitações conhecidas.
14. Antes de adicionar uma dependência, verifique manutenção,
    compatibilidade Linux e suporte à versão estável atual do Rust.
15. Não considere uma fase concluída apenas porque o projeto compila: os
    critérios de aceitação correspondentes precisam ser testados.

**Primeira tarefa de implementação:** executar somente a Fase 1.
Entregar descoberta real de hardware, capacidades, estado original,
preparação segura de monitor mode, loading na TUI e restauração. Não
iniciar a implementação das fases seguintes antes de validar a fundação.
