# Dados Brisanet

Dados coletados sobre informações gerais da Brisanet de hardware, arquitetura, protocolos e criptografia usados na provedora.

## Equipamento

- Fibra FTTH
  GPON Optical Network Unit - AN5506-02-B - Code: MN000004707B - Version: A

- Roteador

  | Informação            | Especificação / Valor                      |
  | --------------------- | ------------------------------------------ |
  | Fabricante            | TP-Link                                    |
  | Modelo                | BSN1200 (Customizado para a Brisanet)      |
  | Tipo de Dispositivo   | Roteador Wireless Gigabit Dual Band AC1200 |
  | Versão de Hardware    | V1.0                                       |
  | Energia / Alimentação | 12V ⎓ 1A                                   |
  | Homologação ANATEL    | 11253-20-03177                             |
  | Segurança             | WPA2-PSK mista com WPA3                    |

## Dados Gerais

- Senhas: As senhas são geradas automaticamente, o usuário pode solicitar uma nova senha entrando no app, mas a senha é sempre gerada pelo backend da Brisanet em um padrão do tipo alfanumerico de 8 digitos (ex: gnt91hkb), não se sabe se tem um padrão de quantas letras (se elas são apenas minúsculas ou pode conter combinações com letra maiússculas) ou números vai na senha, mas é um alfanumerico de 8 digitos. Máscara possível ?1?1?1?1?1?1?1?1

- Sobre TP-LINK: A TP-Link confirma que roteadores personalizados para provedores vêm com configurações específicas do ISP e que as credenciais podem ser definidas ou gerenciadas pela operadora; não existe senha administrativa universal nesses equipamentos.

- ACS Servidor que guarda o inventário de roteadores e envia comandos remotos. O TR-069 usa XML/SOAP entre o ACS e o CPE.

- PMKID É um identificador derivado de material criptográfico relacionado à PMK. Em determinadas implementações e condições, ele pode ser observado em informações de autenticação e associação.

- EAPOL É um protocolo utilizado em processos de autenticação e estabelecimento de chaves. Não deve ser tratado como sinônimo de PMKID.

- RSN IE É um elemento de informação que descreve parâmetros de segurança da rede, incluindo capacidades e suites criptográficas.

- WPS É um mecanismo de configuração e provisionamento. Sua presença não significa automaticamente que a rede esteja vulnerável.

- Protocolos ISP TP-LINK
  - TR-069/CWMP: protocolo clássico de gerenciamento remoto entre o CPE e um ACS; permite provisionamento e configuração remota.
  - TR-369/USP: sucessor mais moderno do TR-069, usado em soluções de nuvem para ISPs.
  - TR-181: modelo de dados usado para expor e alterar parâmetros do CPE, incluindo configurações Wi-Fi.
  - TAUC / Aginet Unified Cloud: solução TP-Link para ISPs que combina gerenciamento em nuvem, TR-069/TR-369, APIs e app customizado.
  - TLS: recomendado para proteger a comunicação de gerenciamento entre CPE e ACS.

- Roteadores
  Costuma usar o TP-Link EC220-G5 (ou uma variação da linha Archer/Agile Solution similar), com a firmware customizada da Brisanet.
