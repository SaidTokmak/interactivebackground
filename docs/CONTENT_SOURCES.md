# Günlük içerik kaynakları ve lisans politikası

Günlük içerik widget'ları ağ isteği yapmaz. Uygulamayla gelen küçük, denetlenmiş
paket yerel tarihe göre deterministik olarak döndürülür. Böylece çevrimdışıyken
boş kart, sonradan değişmiş API yanıtı veya kaynağı belirsiz içerik oluşmaz.
Widget'lar varsayılan olarak ekli değildir; kullanıcı katalogdan eklediğinde
etkinleşir ve istediği anda kaldırabilir.

Her kart eser/yazar veya sure/ayet/hadis numarasını, lisans özetini ve kaynak
bağlantısını gösterir. Paket büyütülürken kaynak ve lisans kaydı bu dosyaya
eklenmeden içerik kabul edilmez.

## Şiirler

Yalnızca telif süresi sona ermiş yazarların kamu malı metinleri kullanılır.

- Yunus Emre, *İlim İlim Bilmektir* — [Türkçe Vikikaynak](https://tr.wikisource.org/wiki/%C4%B0lim_%C4%B0lim_Bilmektir)
- Yunus Emre, *Sevelim sevilelim* — [Türkçe Vikikaynak](https://tr.wikisource.org/wiki/Sevelim_sevilelim)
- Fuzûlî, *Beni candan usandırdı* — [Türkçe Vikikaynak](https://tr.wikisource.org/wiki/Beni_Candan_Usand%C4%B1rd%C4%B1)
- Emily Dickinson, *Hope is the thing with feathers* — [English Wikisource](https://en.wikisource.org/wiki/Poems_(Dickinson,_1890)/Life/XXXII)
- William Blake, *Auguries of Innocence* — [English Wikisource](https://en.wikisource.org/wiki/Auguries_of_Innocence)
- Christina Rossetti, *Who Has Seen the Wind?* — [English Wikisource](https://en.wikisource.org/wiki/Sing-Song:_A_Nursery_Rhyme_Book/Who_has_seen_the_wind%3F)

Wikimedia sayfalarındaki özgün kamu malı eserler kamu malı durumlarını korur.
Sayfa sunumu ve katkı metinleri için [Wikimedia Kullanım
Koşulları](https://foundation.wikimedia.org/wiki/Policy:Terms_of_Use) geçerlidir.

## Kur'an metni ve mealler

- Arapça ayetler Tanzil Project'in Simple metninden alınmıştır. Metin CC BY
  3.0 ile sunulur, değiştirilmeden kullanılmalı ve Tanzil'e bağlantı
  verilmelidir: [Tanzil Quran Text License](https://tanzil.net/docs/text_license).
- Türkçe meal, telif süresi sona ermiş Elmalılı Muhammed Hamdi Yazır metnidir.
  Sure metinleri [Türkçe Vikikaynak Kur'an
  dizininden](https://tr.wikisource.org/wiki/Kur%27an) doğrulanır.
- İngilizce meal, kamu malı Marmaduke Pickthall çevirisidir: [The Meaning of the
  Glorious Koran](https://en.wikisource.org/wiki/The_Meaning_of_the_Glorious_Koran).

Tanzil atfı:

> Tanzil Quran Text Copyright (C) 2007-2021 Tanzil Project — CC BY 3.0

Paket ayetleri: İnşirâh 94:5, Tâhâ 20:114 ve İhlâs 112:1. Arapça metin
normalleştirilmez, sadeleştirilmez veya uygulama tarafından çevrilmez.

## Hadisler

Modern hadis tercümelerinin yeniden dağıtım izni her koleksiyonda açık değildir.
Bu nedenle paket yalnızca kamu malı klasik Arapça kaynak metninin kısa kısmını,
kanonik eser/numara bilgisini ve doğrulama bağlantısını içerir. Uygulama kendi
çevirisini üretmez; çeviri ve bağlam için kullanıcı kaynak sayfasını açar.

- Sahih al-Bukhari 1 — [kaynak](https://sunnah.com/bukhari:1)
- Sahih Muslim 55a — [kaynak](https://sunnah.com/muslim:55a)
- Sahih Muslim 223 — [kaynak](https://sunnah.com/muslim:223)
- Sahih al-Bukhari 2989 — [kaynak](https://sunnah.com/bukhari:2989)
- Sahih al-Bukhari 13 — [kaynak](https://sunnah.com/bukhari:13)

Bağlantılı sitenin modern çevirisi veya sayfa içeriği uygulama paketine
kopyalanmaz. Kaynak düğmesi yalnızca kullanıcının varsayılan tarayıcısında ilgili
doğrulama sayfasını açar.

## Günlük seçim ve gizlilik

Seçim anahtarı yerel takvim günüdür; içerik türleri ayrı sabit ofsetler kullanır.
Saat dilimi içinde gün değiştiğinde kart değişir. Kullanıcı kimliği, telemetri,
konum veya ağ tabanlı kişiselleştirme kullanılmaz. Kaynak düğmesine basılması
dışında ağ erişimi yoktur.
