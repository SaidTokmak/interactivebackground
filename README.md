# interactivebackground

interactivebackground, masaüstünü gerektiğinde sakin bir görev alanına dönüştüren Tauri 2 uygulamasıdır.

## İlk kilometre taşı

- React + TypeScript yönetim ekranı
- Odak ve Kanban wallpaper önizlemeleri
- Rust üzerinde görev modeli ve doğrulama
- Tauri komutlarıyla görev listeleme, ekleme, tamamlama, taşıma ve silme
- SQLite üzerinde kalıcı görev deposu ve Rust birim testleri
- Ayrı `control` ve `wallpaper` Tauri pencereleri
- Rust tarafından yayınlanan `tasks-changed` olayıyla pencere senkronizasyonu
- SQLite üzerinde kalıcı wallpaper şablonu, saydamlık ve etkileşim ayarları
- `settings-changed` olayıyla iki yönlü ayar senkronizasyonu
- İşletim sisteminden okunan çoklu monitör listesi ve kalıcı hedef ekran seçimi
- Fiziksel monitör koordinatlarına göre wallpaper pencere yerleşimi
- Windows'ta gerçek `WorkerW` masaüstü katmanına bağlanma ve güvenli ayrılma
- WorkerW bulunamazsa normal `always-on-bottom` pencereye otomatik geri dönüş
- WorkerW görüntüleme modu ile tıklanabilir top-level etkileşim modu arasında geçiş
- Sistem tepsisi menüsü ve çift tıklamayla yönetim penceresini geri açma
- `Ctrl+Alt+Space` global kısayoluyla sakin/etkileşim modu geçişi
- Explorer yeniden başladığında kontrollü süreç yenileme ve otomatik WorkerW kurtarma
- Kullanıcı aktivitesine göre ayarlanabilir otomatik sakin moda dönüş
- Windows oturum açılışında isteğe bağlı ve tray'e gizli otomatik başlatma
- Uygulama arayüzü, executable ve installer genelinde ortak marka ikonu
- Sistem temasını canlı izleyen kalıcı açık/koyu görünüm seçimi
- Sistem dilini canlı izleyen, SQLite'ta kalıcı Türkçe/İngilizce dil seçimi
- Yönetim ekranı, wallpaper, tarihler, hata mesajları ve tray menüsü için ortak yerelleştirme
- Monitör başına kalıcı hazır tema veya kullanıcı arka planı seçimi
- JPG, PNG ve WebP görseller için kapla/sığdır/uzat, karartma ve blur ayarları
- Folded Horizon, Midnight, Graphite ve Ember hazır arka plan koleksiyonu

## Geliştirme

```powershell
npm install
npm run tauri dev
```

Yalnızca frontend'i tarayıcıda açmak için:

```powershell
npm run dev
```

## Kontroller

```powershell
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

## Windows kurulum paketleri

```powershell
npm run build:desktop
```

Bu komut iki dağıtım biçimi üretir:

- NSIS `.exe`: Türkçe/İngilizce dil seçimi sunan, mevcut kullanıcıya kurulan
  standart kurulum paketi.
- WiX `.msi`: Türkçe ve İngilizce kurumsal dağıtım paketleri.

Çıktılar `src-tauri/target/release/bundle/nsis` ve
`src-tauri/target/release/bundle/msi` klasörlerine yazılır. Geliştirme sürümü
henüz kod imzalama sertifikasına sahip değildir; bu nedenle Windows ilk
çalıştırmada bilinmeyen yayıncı uyarısı gösterebilir.

## Teknik kayıtlar

Geliştirme sırasında çözülen önemli hatalar ve proje sonu raporuna aktarılacak
teknik kararlar [teknik olay günlüğünde](docs/TECHNICAL_INCIDENTS.md) tutulur.
Planlanan ürün aşamaları ve kabul ölçütleri [ürün yol haritasında](docs/ROADMAP.md)
yer alır.

## Sıradaki adımlar

1. Sınırlandırılmış sürüklenebilir widget yerleşim motorunu geliştirmek
2. Pomodoro ve kaynaklı günlük içerik widget'larını eklemek

## Pencere mimarisi

Her iki pencere aynı React bundle'ını yükler. `App.tsx`, Tauri pencere etiketini
okuyarak `ControlWindow` veya `WallpaperWindow` bileşenini seçer. Bir görev
değiştiğinde Rust önce SQLite yazımını tamamlar, ardından bütün pencerelere
`tasks-changed` olayı yayınlar. Her pencere bu sinyali alınca güncel görevleri
yeniden SQLite'tan ister.

Wallpaper ayarları `app_settings` tablosunda tek satır olarak tutulur. Ayar
değişiklikleri de aynı invalidation yaklaşımıyla `settings-changed` olayı
üzerinden bütün pencerelere bildirilir.

Monitör seçimi ad, fiziksel konum ve çözünürlükten oluşturulan bir anahtarla
`monitor_id` sütununda saklanır. Wallpaper gösterilmeden önce native fullscreen
kapatılır; pencereye seçilen monitörün `PhysicalPosition` ve `PhysicalSize`
değerleri uygulanır.

Windows'ta `desktop_host.rs`, wallpaper HWND'sini Explorer'ın `WorkerW`
penceresine child olarak bağlar. Bağlanmadan önce eski parent ve `GWL_STYLE`
değerleri kaydedilir. WorkerW, Microsoft'un kararlı bir uygulama API'si
olmadığı için keşif veya bağlanma başarısız olursa wallpaper normal pencere
modunda çalışmaya devam eder.

WorkerW katmanı masaüstü ikonlarının arkasındadır ve Explorer fare olaylarını
önce yakalar. Bu nedenle WorkerW sakin görüntüleme modudur. `Düzenleme modu`
açıldığında aynı wallpaper penceresi önce WorkerW'den ayrılır, seçilen monitörün
fiziksel sınırlarına top-level ve always-on-top olarak yerleşir. WebView bu
durumda fare/klavye olaylarını alabilir. Düzenleme kapatılınca pencere yeniden
WorkerW altına bağlanır; React state'i, Rust komutları ve SQLite verisi korunur.

Wallpaper kapatıldığında WorkerW'den ayrılmış gizli bir native pencere
bırakılmaz. İkinci monitörlerde DWM/Explorer kompozisyon izi oluşmaması için
wallpaper `WebviewWindow` tamamen yok edilir ve Windows masaüstü katmanı mevcut
monitör duvar kâğıtları korunarak yenilenir. Wallpaper tekrar açılırken aynı
etiketle yeni bir pencere oluşturulur; kalıcı görev ve ayar verileri SQLite'tan
yeniden yüklenir.

Explorer yeniden başlatılırsa WorkerW ile birlikte child wallpaper HWND'si de
Windows tarafından yok edilir. interactivebackground watchdog'u geçersiz native bağlantıyı
algılar, bir kurtarma işareti yazar ve süreci kontrollü olarak yeniden başlatır.
Yeni süreç Tauri `RunEvent::Ready` aşamasında kalıcı ayarları SQLite'tan okuyup
wallpaper'ı yeni Explorer sürecinin WorkerW katmanına otomatik bağlar.

Etkileşim modu açıldığında Rust son kullanıcı aktivitesinin zamanını tutar.
Wallpaper WebView'i fare ve klavye aktivitesini en fazla 15 saniyede bir native
katmana bildirir. Ayarlanan 1, 5, 10 veya 15 dakikalık süre boyunca aktivite
olmazsa ayar SQLite'ta sakin moda çevrilir ve pencere otomatik olarak WorkerW
katmanına geri bağlanır. Bu davranış yönetim ekranından tamamen kapatılabilir.

Uygulamanın görünen adı, npm paketi, Rust crate'i ve executable adı
`interactivebackground` olarak değiştirilmiştir. Mevcut geliştirme verilerini
kaybetmemek için Tauri identifier `com.flowdesk.app` ve mevcut `flowdesk.db`
dosya adı geriye dönük uyumluluk amacıyla şimdilik korunur; bunlar kullanıcıya
görünen marka adı değildir.

Windows otomatik başlangıç seçeneği Tauri autostart eklentisinin gerçek sistem
kaydını okuyup değiştirir. Etkinleştirilen kayıt executable'ı `--hidden`
argümanıyla çalıştırır. Uygulama bu argümanı gördüğünde kontrol penceresini
setup aşamasında gizler; tray, global kısayol ve arka plan servisleri çalışmaya
devam eder. Manuel açılışta yönetim penceresi normal şekilde gösterilir.
