# Hitodenashi \~Pear Alarm\~

![Hitodenashi logo](doc/img/image.png)

梨農家で利用できそうな防犯ブザーを開発してみました。
きっかけになったニュースはこちらです。

<iframe width="560" height="315" src="https://www.youtube.com/embed/QepSUQ5gga4?si=cp9hxLfi0Drh45GF&amp;controls=0" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## Hardware

- RP2040
- Baker link.Dev
- GROVE - Speaker（アンプ内蔵）
- クリップ式の入力スイッチ

### GPIO

- GPIO0: クリップ状態の入力
- GPIO28: スピーカー出力

## Firmware

Rust で実装しています。
GPIO の High / Low を高速に切り替えて簡単な矩形波を生成し、GROVE Speaker から警報メロディを鳴らしています。
現在のファームウェアでは、GPIO0 の状態を監視し、クリップが外れた状態になると警報メロディを再生します。

## How it works

1. GPIO0 を pull-down 入力として監視します。
2. クリップが外れて GPIO0 が Low になると、警報メロディを再生します。
3. GPIO28 を出力にして、High / Low を短い間隔で切り替えます。
4. その波形を GROVE Speaker に送り、耳に残る警報音にします。

## Story

梨泥棒事件を見て、「守られる側」であるはずの梨が、自分で自分を守ったら面白いのでは、と思ったのが始まりです。

見た目は梨。
機能は防犯ブザー。
発想はふざけているのに、動きは意外とまじめ。

それが Hitodenashi です。

## Concept

- 見た目はかわいい
- 中身はかなり物騒
- でもちゃんと動く
- 梨が自分を守る、という一発ネタを本気で作る

---

個人の電子工作プロジェクトです。
完璧さより、形にしてみること優先して作成をしました。

