# FilumWare

Ferramenta para hacking de wi-fi focados nos padrões de provedores (Ex: Brisanet). Usuário consegue varrer as redes próximas, seleciona uma rede alvo, coleta-se e salva o máximo de informações sobre a rede (modelo do roteador, MACs, frequência, canal etc.), depois monitora-se ela enquanto se faz um envio de pacotes de associação para forçar o roteador a responder com o PMKID, uma vez com tudo isso em mãos é gerado arquivos nas extensões para as ferramentas (.pcap; .pcapng; .txt; .rule; .json), esses arquivos ficam em uma pasta criada automaticamente quando o usuário selecionou a rede alvo, o nome da pasta fica com um HEX para identificação gerados com informações do BSSID da rede e a timedate, com isso a ferramenta cria todos os arquivos com o mesmo nome e sabe qual a rede está em "target" fazendo a pipeline conhecer apenas o arquivos da rede alvo, com todos os arquivos gerados usamos um modelo de IA para gerar um diagnostico, basicamente ela ler todos os arquivos e gera outro arquivo .json que informa qual o tipo de segurança que vai ser quebrada, qual o formato ideal a ser usado no ataque, se é WPS, Mask, Wordlist, e dentro desse arquivo conforme o tipo de segurança e formato ela gera a máscara ideal pro ataque ou a wordlist, assim quando passamos para a última fase, é mostrado esse relatório ao usuário e as opções de como ele deseja atacar (se ele escolhe "mask" é passado o comando a máscara criada), dessa forma é iniciado o ataque e o usuário só aguarda o processo da quebra de senha (não é garantido pode falhar), o usuário acompanha tudo isso em tela em tempo real.

## Flow

1. Varredura: Varre as 50 redes com sinal mais forte próximas, mostra pro usuário o máximo de detalhes possíveis.
2. Coleta: Monitora e escuta o ar da rede alvo selecionada na etapa de varredura, coleta todos os dados possíveis da rede interagindo com o roteador.
3. Captura: Usamos todos os dados salvos sobre a rede alvo para começar capturar tudo nela, desde dispositivos conectados ao envio de pacotes de associação para forçar o roteador a responder com o PMKID (presente na resposta do RSN IE ou Key Data),também é feito um teste para saber se a rede tem WPS ativo para liberar a opção de ataque via WPS, e assim que temos o sinal valido que conseguimos o que é necessário para um ataque passado para próxima etapa.
4. Pacotes: Geramos o arquivo em todos os formatos possíveis para ser usados, aqui usamos o poder do Rust com uma pipeline assíncrona/paralela para processar o arquivo da melhor forma possível sem perder nada, com consumo de memória controlado para arquivos pesados a >200MB/s, mantendo sua integridade e principalmente o máximo de dados validos possíveis, processa sem trava de tamanho (No EAPOL size gate).
5. Quebra: Pegamos os arquivos gerados e processamos eles varrendo locais onde o PMKID pode estar escondido (frames de Associação, Reassociação, Beacons com firmware bugado, Auth frames, etc.), depois usamos IA para decidir qual a melhor máscara ou método para quebra da senha com base em todos os dados coletados da rede (frequências, modelo de roteador, protocolos), no caso da Brisanet ela usar um padrão alfanúmerico de 8 digitos misto que as vezes vem com números e letras (as vezes todas minúsculas, as vezes com umas minúsculas e maiúsculas e vise e versa).

## Layers

Dividimos em 3 camadas:

- Storage: Varremos, coletamos, geramos e salvamos os dados de tudo sobre a rede.
- Parsing: Usamos tudo que temos da rede principalmente os arquivos gerados contendo o PMKID (presente na resposta do RSN IE ou Key Data), aplicamos uma IA para gerar um diagnostico geral sobre a rede e o melhor formato para o ataque.
- Hacking: Com o diagnostico e tudo em mãos damos inicio ao ataque e quebra da senha usando o melhor método/técnica, aplicando combinações e máscaras mais compativeis com cenário da rede alvo.

## Tools

- Interface Visual (TUI): ratatui (sucessor espiritual do tui-rs) combinando com crossterm para desenhar dashboards dinâmicos no terminal com gráficos de sinal, tabelas de BSSIDs e barras de progresso de captura.

- Concorrência e Assincronia: tokio para gerenciar threads assíncronas separadas (uma thread controlando a varredura/escuta de pacotes, outra controlando o estado da TUI e outra gerando os comandos externos).

- Interação com o Sistema: O Rust chamará os binários de sistema otimizados (hcxdumptool / hashcat) via std::process::Command ou fará chamadas diretas a sockets via bindings se necessário, mantendo o controle total do ciclo de vida do processo.
