# interactivebackground ürün yol haritası

Bu belge ürün geliştirme sırasını, özelliklerin birbirine olan bağımlılıklarını
ve kabul ölçütlerini tanımlar. Sıra, her aşamanın bir sonraki aşama için sağlam
bir temel oluşturacağı şekilde düzenlenmiştir.

## Faz 1 — Tema sistemi ve tasarım altyapısı

Durum: Tamamlandı — 15 Temmuz 2026

İlk sırada arayüzün tamamını ortak tasarım token'larına geçirmek vardır. Bu
sayede daha sonra eklenecek ayar, widget ve karşılama ekranları aynı tema
sistemini kullanabilir.

- `Sistem`, `Açık` ve `Koyu` görünüm seçenekleri.
- İşletim sistemi tema değişikliğini canlı takip eden `Sistem` modu.
- Seçimin SQLite'ta kalıcı tutulması ve iki pencereye anında yayınlanması.
- Yönetim ekranı ile wallpaper widget'larının ayrı okunabilirlik değerlerine
  sahip olması; açık/koyu tema değişse bile seçilen arka planı boğmaması.
- Seçilen Folded Horizon marka paletinden türetilen nötr arayüz renkleri:
  grafit, gece laciverti, kırık beyaz ve sınırlı mat mercan vurgu.
- Kontrast, klavye odağı ve azaltılmış hareket tercihleri için erişilebilirlik
  kontrolü.

Kabul ölçütü: Uygulama yeniden başlatıldığında tema korunur ve her iki pencere
aynı anda doğru temaya geçer.

## Faz 2 — Yerelleştirme ve dil seçimi

Durum: Tamamlandı — 15 Temmuz 2026

Metinler bileşenlerden çıkarılıp anahtar tabanlı dil kaynaklarına taşınacaktır.
İlk teslimat Türkçe ve İngilizce olur; altyapı yeni bir JSON kaynak dosyasıyla
yeni dil eklenebilecek şekilde hazırlanır.

- Ayarlarda dil seçici ve isteğe bağlı `Sistem dili` seçeneği.
- Dil tercihinin SQLite'ta tutulması ve pencereler arasında senkronizasyonu.
- İlk paket: Türkçe (`tr`) ve İngilizce (`en`).
- Sonraki paketler: Almanca, Fransızca ve İspanyolca.
- RTL altyapısı doğrulandıktan sonra Arapça dil paketi.
- Tarih, saat, sayı ve çoğul ifadelerinde tarayıcının `Intl` API'lerini kullanma.
- Eksik çeviride İngilizce fallback ve geliştirme sırasında eksik anahtar
  kontrolü.
- Tray menüsü ve native hata metinleri dahil Rust tarafındaki kullanıcıya
  görünen metinlerin de yerelleştirilmesi.

Kabul ölçütü: Dil değiştirmek yeniden başlatma gerektirmez; yönetim ekranı,
wallpaper ve tray aynı dili gösterir.

## Faz 3 — Kullanıcı arka planları ve hazır temalar

Durum: Tamamlandı — 15 Temmuz 2026

- Kullanıcının dosya seçiciyle JPG, PNG veya WebP arka plan seçebilmesi.
- `Kapla`, `sığdır` ve `uzat` görüntüleme seçenekleri.
- Her monitör için bağımsız arka plan ve tema tercihi.
- Uygulamayla gelen küçük, lisansı açık bir hazır tema koleksiyonu.
- Widget okunabilirliği için isteğe bağlı karartma, blur ve renk tonu katmanı.
- Dosya silinir veya erişilemez olursa güvenli varsayılan temaya dönüş.
- Kullanıcı görsellerinin cihaz dışına gönderilmemesi.

Kabul ölçütü: Seçilen görsel yeniden başlatma ve Explorer kurtarması sonrasında
korunur; farklı çözünürlük ve ekran oranlarında doğru ölçeklenir.

## Faz 4 — Sınırlandırılmış widget yerleşim motoru

Durum: Tamamlandı — 15 Temmuz 2026

Bu faz, kullanıcının masaüstü klasörlerinin kapladığı alanla çakışmayan bir
çalışma alanı oluşturmasını sağlar. Windows masaüstü ikonlarının konumunu
belgelenmemiş yöntemlerle otomatik okumak yerine kullanıcı güvenli alanı görsel
olarak belirler; bu yaklaşım Explorer sürümleri arasında daha kararlıdır.

- Yalnızca düzenleme modunda sürükleme ve yeniden boyutlandırma.
- Widget'ın monitör sınırları dışına çıkmasını engelleyen sert sınırlar.
- Minimum/maksimum boyutlar, kenara yapışma ve isteğe bağlı grid.
- Ana görev alanının monitörde istenen konuma taşınması.
- Konum ve boyutların piksel yerine normalize edilmiş koordinatlarla saklanması;
  çözünürlük veya DPI değişince yerleşimin korunması.
- Monitör ve wallpaper şablonu başına ayrı yerleşim.
- Yerleşimi kilitleme, sıfırlama ve varsayılan konuma dönme.
- Ekrandan çıkarılmış monitör için güvenli yerleşim fallback'i.

Kabul ölçütü: Widget hiçbir DPI veya çoklu monitör düzeninde görünür alanın
dışına çıkmaz; kullanıcı klasörlerini boşta bırakacak konumu seçip saklayabilir.

## Faz 5 — Widget altyapısı ve ilk widget'lar

İlk sürümde üçüncü taraf kod çalıştıran açık bir eklenti sistemi yerine,
uygulama içinde tanımlı güvenli widget kataloğu kullanılacaktır.

- Ortak widget modeli: tür, konum, boyut, tema, ayarlar ve görünürlük.
- Widget ekleme, kaldırma, çoğaltma, sıralama ve kilitleme.
- Her widget için tanımlı minimum/maksimum alan gereksinimi.
- İlk widget'lar:
  - Mevcut odak görev listesi ve Kanban.
  - Pomodoro: çalışma/mola süreleri, başlat–duraklat–sıfırla ve bildirim.
  - Saat ve tarih.
- Sayaçların pencere yeniden oluşturulurken kaybolmaması için Rust tarafında
  güvenilir zaman durumu.

Kabul ölçütü: Birden fazla widget sınırlar içinde birlikte çalışır ve wallpaper
penceresi yeniden oluşturulduğunda yerleşim ile sayaç durumu korunur.

## Faz 6 — Günlük içerik widget'ları

- Günün şiiri: yalnızca kamu malı, açık lisanslı veya izinli içerik koleksiyonu.
- Ayet widget'ı: sure/ayet numarası, çeviri adı ve kaynak gösterimi zorunlu.
- Hadis widget'ı: eser, bölüm/numara ve doğrulanabilir kaynak gösterimi zorunlu.
- Dini içeriklerin tamamen isteğe bağlı olması ve kullanıcı tarafından içerik
  paketi/çeviri seçilebilmesi.
- Çevrimdışı içerik paketleri veya güvenilir API sağlayıcısı için cache,
  bağlantı hatası ve günlük yenileme politikası.
- Dil ve bölgeye göre uygun içerik; kaynağı belirsiz rastgele metin gösterilmez.

Kabul ölçütü: Her içerik kartı kaynağını açıkça gösterir, internet kesilince son
doğrulanmış içerik korunur ve kullanıcı özelliği tamamen kapatabilir.

## Faz 7 — İlk kullanım ve kişiselleştirme akışı

- İlk açılışta dil ve tema seçimi.
- Hedef monitör, arka plan, widget alanı ve global kısayol tanıtımı.
- Autostart seçeneğinin açık rızayla sunulması; varsayılan olarak kapalı kalması.
- Örnek widget yerleşimi ve tek tıkla boş başlangıç seçeneği.
- Karşılama akışını ayarlardan tekrar açabilme.

## Faz 8 — Yayın ve güncelleme

- Tauri updater entegrasyonu ve imzalı güncelleme manifesti.
- Windows kod imzalama sertifikası.
- GitHub Actions ile test, paketleme, checksum ve GitHub Release otomasyonu.
- Legacy `com.flowdesk.app` identifier ve `flowdesk.db` için kontrollü veri
  taşıma planı.

## Ürün kararları ve sınırlar

- “Birçok dil” tek seferde makine çevirisiyle yayınlanmayacak; önce altyapı ve
  iki doğrulanmış dil, ardından test edilen dil paketleri gelecektir.
- Şiir, ayet ve hadis kaynakları içerik kalitesi ile telif/lisans kontrolünden
  geçmeden uygulamaya eklenmeyecektir.
- İlk widget sistemi uygulama içi katalogla sınırlıdır. Üçüncü taraf widget kodu
  çalıştırmak güvenlik, izin ve sürümleme modeli tasarlanmadan açılmayacaktır.
- Masaüstü ikon konumlarını Explorer'ın belgelenmemiş iç yapısından okumak
  yerine kullanıcı kontrollü güvenli alan kullanılacaktır.
- Bütün yerleşimler çoklu monitör, negatif koordinat ve farklı DPI koşullarında
  test edilecektir.

## Önerilen bir sonraki çalışma

Faz 5'e geçilmelidir: mevcut görev alanını ilk katalog öğesine dönüştüren ortak
widget modeli kurulmalı; ekleme, kaldırma, görünürlük, sıralama ve bağımsız
yerleşim kuralları hazırlanmalıdır. Bu temel üzerinde ilk yeni öğe olarak
kalıcı zaman durumuna sahip Pomodoro geliştirilmelidir.
