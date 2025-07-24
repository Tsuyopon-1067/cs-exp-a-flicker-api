# cs-exp-a-flicker-api

[静岡大学情報学部情報科学科情報科学実験Aの最終課題](https://ohkilab.github.io/SU-CSexpA/content/part2/part2_final_assignment/regulation.html)のAPIサーバです．

## 使用方法

前処理をしてから，サーバを起動します．先にビルドをしておくと試行錯誤がスムーズです．いずれのコマンドも`server`ディレクトリに入ってから実行します．

### Rustインストール

[公式サイト](https://www.rust-lang.org/ja/tools/install)を参考にして下さい．
MacやLinuxでは以下のコマンドによりインストールできます．推奨インストールをすればパスが通るはずなので，`source`コマンドで`./bashrc`や`.zshrc`を読み込み直しましょう．
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### ビルド

実行ファイルにアクセスしやすくするために，ビルドされた実行ファイルを`server`ディレクトリ直下に移動します．

```bash
$ cargo build --release && mv target/release/server .
```

### 前処理

以下のどちらでも良いです．`tag.csv`と`geotag.csv`は，`server`ディレクトリの直下に配置しておきます．
メモリを多く消費するので，Raspberry Piでの実行は非推奨です．処理時間は MacBook Air M4 で約90秒です．
前処理後に`tag_photodata_map.bin`というファイルが生成されますが，サーバが読み込むので削除・移動・リネームはしないようにして下さい．

```bash
$ ./server --prepare tag.csv geotag.csv
$ ./server -p tag.csv geotag.csv
```

### サーバ

HTTP/2接続のための自己署名SSL/TLS証明書を生成します．`<IP_ADDRESS>`はサーバのIPアドレス（`xx.xx.xx.xx`）を入力します．ポート番号は不要です．
```bash
$ openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes -subj "/C=JP/ST=Tokyo/L=Chiyoda/O=Example Inc./OU=IT/CN=<IP_ADDRESS>"
```

あとはサーバを起動するだけです．
```bash
$ ./server
```

## 実装
### 方針
- 全てのデータを主記憶に格納するオンメモリ方式

### 前処理
- あらかじめ，各tagから写真データ100件のHashMapを構築しておき，バイナリファイルとして保存する．
    1. 各tagからidへのmapを構築．
    2. 各idから写真データへのmapを構築．
    3. 各tagに対応する写真データ配列を構築．
    4. 写真データ配列は長さが100を超えないように適宜カットする．
    5. 写真データ配列をjson文字列化し，gzipに圧縮．
    6. 各tagから写真データのjson文字列のgzipデータへのmapを構築．
    7. 構築されたmapをバイナリファイルとして保存．
### サーバ
- 前処理で構築しておいたmapを読み込み，サーバを立ち上げて待機する．
- リクエストの種類によらずgzip形式でレスポンスを返す．
- レスポンスヘッダもgzip形式で圧縮する．