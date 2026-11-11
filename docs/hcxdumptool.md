# Guia Completo de Parâmetros e Flags: hcxdumptool (v8.x)

## hcxdumptool

O hcxdumptool é uma ferramenta de nível de enlace (Layer 2) voltada para auditoria de redes sem fio. Ela atua ativamente para capturar elementos de autenticação do protocolo WPA/WPA2/WPA3 (como PMKIDs e handshakes EAPOL). Contudo, ela não realiza conversão de hashes nem quebra de senhas para isso, trabalha integrada com o restante do pacote hcxtools (principalmente o hcxpcapngtool).

### Parâmetros

| Parâmetro / Flag | Tipo de Valor | Objetivo | Propósito | Quando Usar |
| --- | --- | --- | --- | --- |
| **`-i`**, **`--interface`** | `<interface>` | Especificar a interface sem fio. | Informar ao programa qual placa de rede Wi-Fi em modo monitor deve ser utilizada para captura e injeção de pacotes. | **Sempre obrigatório** em qualquer operação de captura ou varredura. |
| **`-w`**, **`--pcapng`** | `<arquivo.pcapng>` | Salvar a captura em arquivo. | Escrever todos os quadros e metadados de rádio recebidos no formato padrão `.pcapng`. | **Sempre obrigatório** para registrar as capturas que serão convertidas posteriormente pelo `hcxpcapngtool`. |
| **`-c`**, **`--channel`** | `<canal>` | Travar a placa em um canal. | Fixar a escuta e transmissão em uma única frequência/canal (ex: `-c 1`, `-c 36`), desativando o *channel hopping*. | Usar quando o alvo já foi identificado para evitar perda de pacotes trocando de frequência. |
| **`--bpfc`** | `="expressão"` | Compilar regra BPF. | Traduzir uma expressão de filtro em texto (estilo `pcap-filter`/`tcpdump`) para formato binário de máquina BPF. | Usar na fase de preparação para gerar o arquivo `.bpf` restrito ao BSSID do alvo. |
| **`--bpf`** | `=<arquivo.bpf>` | Aplicar filtro BPF binário. | Carregar o arquivo `.bpf` compilado e filtrar o tráfego em tempo real no kernel. | Usar na execução da captura para garantir que o programa afete e registre apenas a rede especificada. |
| **`--rcascan`** | `=active` \| `=passive` | Executar varredura de redes. | Listar em tempo real todos os Access Points ao alcance, exibindo BSSID, ESSID, canal, frequência e RSSI. | Usar na etapa inicial de reconhecimento para descobrir os BSSIDs e canais disponíveis. |
| **`-F`** | N/A | Aplicar filtros no `rcascan`. | Otimizar a exibição da tabela de varredura ativa/passiva no terminal. | Usar em conjunto com `--rcascan` para filtrar informações irrelevantes na tela. |
| **`--enable_status`** | `<máscara>` | Controlar mensagens do terminal. | Habilitar a exibição detalhada de estatísticas, pacotes EAPOL, Probe Requests e eventos de status via bitmask (ex: `15`). | Usar durante a captura para monitorar em tempo real a recepção de PMKID e handshakes. |
| **`--exit_on_eapol`** | N/A | Interromper captura ao obter sucesso. | Finalizar automaticamente a execução do programa assim que um quadro EAPOL válido for capturado. | Usar em scripts automatizados para encerrar o processo logo que o hash da senha for obtido. |
| **`--associationmax`** | `<número>` | Limitar solicitações de associação. | Definir a quantidade máxima de tentativas de associação enviadas ao Access Point. Valor `0` desativa todas. | Usar para controlar o nível de agressividade da ferramenta ou evitar bloqueios por IPS/IDS da rede. |
| **`--beacontx`** | `<0 ou 1>` | Controlar transmissão de Beacons. | Habilitar (`1`) ou desabilitar (`0`) a injeção ativa de quadros Beacon simulados pela placa de rede. | Usar `0` para capturas totalmente passivas ou `1` para forçar clientes a tentar conexão. |
| **`--proberesponsetx`** | `<0 ou 1>` | Controlar resposta a Probe Requests. | Habilitar (`1`) ou desabilitar (`0`) o envio de Probe Responses para solicitações não direcionadas. | Usar `0` quando quiser evitar que a sua placa responda a buscas de clientes genéricos ao redor. |
| **`--rds`** | `<nível>` | Definir fonte de dados de rádio. | Configurar como a ferramenta interpreta e extrai os metadados dos cabeçalhos Radiotap da interface. | Usar quando operar com adaptadores Wi-Fi que utilizam drivers customizados ou legados. |
| **`--rdt`** | N/A | Desativar ajuste de terminal. | Impedir que o programa verifique o tamanho da tela (`TIOCGWINSZ`), mantendo o layout fixo. | Usar ao rodar o script dentro de subshells, containers Docker ou sem terminal TTY atrelado. |
| **`-h`** | N/A | Exibir ajuda resumida. | Mostrar a lista rápida de opções essenciais de linha de comando. | Usar para consulta rápida da sintaxe das flags no terminal. |
| **`--help`** | N/A | Exibir ajuda completa. | Exibir o manual detalhado com exemplos práticos, arquitetura BPF e resolução de problemas. | Usar para entender detalhes operacionais e exemplos avançados de uso. |
| **`-v`**, **`--version`** | N/A | Exibir versão e capacidades. | Mostrar a versão exata da ferramenta, bibliotecas vinculadas e suporte a compilador BPF ativado. | Usar para verificar a compatibilidade de versão ou ao reportar problemas no repositório. |

### Exemplos

- **Varredura e Identificação:**

   ```bash
      sudo hcxdumptool -i wlp1s0 --rcascan=active -F
   ```

- **Compilação do Filtro BPF para o BSSID Alvo:**

   ```bash
      hcxdumptool --bpfc="wlan addr3 A0:A0:A0:A0:A0:A0" > target.bpf
   ```

- **Captura Direcionada do PMKID/Handshake:**

   ```bash
      sudo hcxdumptool -i wlp1s0 -w capture.pcapng --bpf=target.bpf -c $CHANNEL --enable_status=15
   ```

> **⚠️ Observações:**
>
