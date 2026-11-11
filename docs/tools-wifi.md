# Guia de Análise e Auditoria de Redes Sem Fio (Wi-Fi)

Este documento reúne a análise técnica das principais ferramentas de auditoria de segurança Wi-Fi, abordando o funcionamento, casos de uso, vantagens, limitações e exemplos práticos acoplados a cada ferramenta.

---

## 1. Suíte Aircrack-ng (Airmon-ng, Airodump-ng, Aireplay-ng)

### O que é e o que faz
A suíte **Aircrack-ng** é um conjunto clássico de ferramentas para monitoramento, captura de pacotes, injeção de tráfego e quebra de chaves em redes 802.11 (WPA/WPA2/WEP).
* **airmon-ng:** Gerencia a interface de rede, habilitando/desabilitando o modo monitor e finalizando processos que interferem na captura.
* **airodump-ng:** Captura quadros 802.11 e exibe informações em tempo real sobre APs (Access Points) e clientes conectados.
* **aireplay-ng:** Realiza injeção de pacotes (como desautenticação de clientes) para forçar o re-handshake WPA.

### Objetivo e Cenários de Uso
* **Objetivo:** Capturar o Handshake EAPOL (4-way handshake) através da desautenticação ativa de clientes conectados.
* **Cenários de Uso:** Ambientes onde o suporte ao protocolo WPS está desativado ou onde ferramentas como `hcxdumptool` falham devido a limitações do driver da placa de rede.

### Prós e Contras
* **Prós:**
  * Extremamente estável e compatível com a maioria dos chipsets Wi-Fi.
  * Padrão da indústria para injeção e desautenticação de clientes.
* **Contras:**
  * Exige que haja pelo menos um cliente ativo conectado ao AP para capturar o handshake EAPOL tradicional.
  * Requer intervenção manual em múltiplos terminais (um para monitorar, outro para injetar pacotes).

### Exemplo de Uso
```bash
sudo airmon-ng check kill
sudo airmon-ng start wlp1s0 
sudo airodump-ng -c 7 --bssid 00:E0:20:81:76:73 -w captura_alvo wlp1s0mon
sudo aireplay-ng --deauth 10 -a 00:E0:20:81:76:73 wlp1s0mon
hcxpcapngtool -o hash.22000 captura_alvo-01.cap
```

---

## 2. Hcxtools (hcxdumptool & hcxpcapngtool)

### O que é e o que faz
O **hcxdumptool** é uma ferramenta moderna de captura de quadros em Camada 2 focada na obtenção de material criptográfico (PMKID e EAPOL) diretamente de pontos de acesso. O **hcxpcapngtool** processa e converte os arquivos capturados (`.pcapng`) para o formato unificado do Hashcat (`.22000`).

### Objetivo e Cenários de Uso
* **Objetivo:** Captura passiva ou semipassiva de PMKID (sem necessidade de clientes conectados) ou handshakes EAPOL completos.
* **Cenários de Uso:** Auditorias modernas em redes WPA2/WPA3 onde se deseja capturar hashes de forma automatizada para processamento em GPU via Hashcat.

### Prós e Contras
* **Prós:**
  * Permite capturar o PMKID instantaneamente sem precisar esperar ou desconectar clientes.
  * Interface moderna e integração direta com o ecossistema do Hashcat (`hash.22000`).
  * Permite o uso de filtros BPF (`--bpfc`) para focar estritamente no BSSID de interesse.
* **Contras:**
  * Exige drivers com suporte a injeção avançada (frequentemente falha em placas integradas comuns com o erro `failed to arm interface`).
  * Exige que a placa de rede permaneça no modo `managed` em versões recentes da ferramenta.

### Exemplo de Uso
```bash
sudo systemctl stop NetworkManager
sudo airmon-ng check kill
sudo ip link set wlp1s0 down
sudo iw wlp1s0 set type managed
sudo ip link set wlp1s0 up
hcxdumptool --bpfc="wlan addr3 00:E0:20:81:76:73" > target.bpf
sudo hcxdumptool -i wlp1s0 -w capture.pcapng --bpf=target.bpf -c 7
hcxpcapngtool -o hash.22000 capture.pcapng
hashcat -m 22000 -a 3 --force -d 1 -O -w 3 -1 '?l?u?d' hash.22000 "?1?1?1?1?1?1?1?1"
```

---

## 3. Reaver & PixieWps

### O que é e o que faz
* **Reaver:** Ferramenta desenvolvida para explorar a vulnerabilidade de PIN no protocolo WPS (Wi-Fi Protected Setup) via força bruta online.
* **PixieWps (Pixie-Dust Attack):** Ferramenta offline que explora a baixa entropia na geração de números pseudo-aleatórios (PRNG) de determinados chipsets de roteadores, calculando o PIN WPS a partir de poucas trocas de mensagens.

### Objetivo e Cenários de Uso
* **Objetivo:** Recuperar a chave de rede (WPA/WPA2) em texto plano explorando a falha do mecanismo de autenticação PIN do WPS, ignorando a complexidade da senha do Wi-Fi.
* **Cenários de Uso:** Roteadores antigos ou firmwares sem proteção contra força bruta em que o recurso WPS permanece habilitado e vulnerável.

### Prós e Contras
* **Prós:**
  * Revela a senha original em texto plano, independente de quão longa ou complexa ela seja.
  * O Pixie-Dust obtém a chave em poucos segundos offline.
* **Contras:**
  * Ineficaz em redes modernas com mecanismo de bloqueio (*WPS Lockout* ou *rate-limiting*).
  * Inútil caso o WPS esteja desativado no painel do roteador.

### Exemplo de Uso
```bash
sudo airmon-ng start wlp1s0
sudo wash -i wlp1s0mon
sudo reaver -i wlp1s0mon -b 00:E0:20:81:76:73 -c 7 -K 1 -vv
```

---

## 4. Wifite

### O que é e o que faz
O **Wifite** é um script de automação em Python que atua como wrapper para diversas ferramentas de auditoria sem fio (Aircrack-ng, Reaver, PixieWps, Hcxdumptool, Tshark). Ele varre o espectro de rádio e aplica ataques em sequência contra os alvos selecionados.

### Objetivo e Cenários de Uso
* **Objetivo:** Automatizar o processo de auditoria (colocar em modo monitor, selecionar alvos, tentar Pixie-Dust, capturar PMKID/Handshake e testar dicionários básicos).
* **Cenários de Uso:** Avaliações rápidas de múltiplos pontos de acesso no ambiente sem a necessidade de executar comandos manuais para cada etapa.

### Prós e Contras
* **Prós:**
  * Interface simples e interativa.
  * Combina múltiplos vetores de ataque (WPS, PMKID, Handshake) em uma única execução.
* **Contras:**
  * Pode gerar falsos positivos ao indicar que o WPS está ativo (lendo apenas a tag nos *Beacons*, mesmo quando o PIN está travado internamente).
  * Não executa ataques em paralelo devido à limitação física de sintonia de canal da placa de rede.

### Exemplo de Uso
```bash
sudo wifite --wps --power 50
```

---

## 5. Bettercap

### O que é e o que faz
O **Bettercap** é um framework modular para reconhecimento de rede, ataques Man-in-the-Middle (MITM), sniffing e manipulação de tráfego. Possui módulos dedicados para auditoria Wi-Fi, Ethernet, Bluetooth Low Energy (BLE) e SDR.

### Objetivo e Cenários de Uso
* **Objetivo:** Oferecer uma plataforma unificada para reconhecimento de rádio, captura de PMKID/Handshakes e análise de tráfego pós-conexão.
* **Cenários de Uso:** Testes de intrusão onde o escopo engloba desde a captura de material criptográfico Wi-Fi até a análise e interceptação de tráfego interno da rede.

### Prós e Contras
* **Prós:**
  * Extremamente rápido, modular e expansível via *caplets*.
  * Permite forçar associações para captura direta de PMKID via terminal interativo.
* **Contras:**
  * Exige privilégios de superusuário (`root`).
  * Curva de aprendizado maior para dominar os comandos do prompt interativo.

### Exemplo de Uso (Captura de PMKID)
```bash
sudo bettercap -iface wlp1s0
wifi.recon on
wifi.recon.channel 7
wifi.assoc 00:E0:20:81:76:73
wifi.recon.bssid 00:E0:20:81:76:73
hcxpcapngtool -o hash.22000 bettercap-wifi-handshakes.pcap
```
