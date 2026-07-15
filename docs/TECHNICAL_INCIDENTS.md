# interactivebackground Teknik Olay Günlüğü

Bu dosya, geliştirme sırasında karşılaşılan önemli hataların proje sonundaki
detaylı teknik rapora aktarılması için kalıcı kayıt olarak tutulur.

## FD-WIN-001 — İkinci monitörde kalan siyah wallpaper penceresi

- Tarih: 14 Temmuz 2026
- Durum: Çözüldü ve gerçek ikinci monitörde doğrulandı
- İlgili alan: Tauri 2, Rust, Win32 WorkerW, çoklu monitör, pencere yaşam döngüsü
- Final rapora dahil et: Evet

### Belirti

Wallpaper kapatıldıktan veya yönetim paneline dönüldükten sonra ikinci
monitörde o zamanki adıyla `Flowdesk Wallpaper` başlıklı siyah bir pencere, başlık çubuğu ve
beyaz çerçeve kalıyordu. Tauri ve Win32 görünürlük sorguları pencereyi gizli
göstermesine rağmen ekrandaki görüntü kaybolmuyordu. Sorun özellikle
`(2560, -247)` başlangıç koordinatına ve `1080x1920` çözünürlüğe sahip dikey
ikinci monitörde yeniden üretildi.

Aynı süreçte yönetim penceresindeki standart kapatma düğmesinin uygulamayı
görev çubuğuna küçültmek yerine sistem tepsisine gizlemesi istendi.

### İlk teşhis ve neden yetersiz kaldığı

İlk yaklaşım, wallpaper penceresini Tauri `hide()` ve Win32
`ShowWindow(SW_HIDE)` çağrılarıyla iki kez gizlemek, ardından WorkerW
bağlantısını kaldırmaktı. `IsWindowVisible` sonucu `false` olduğu için sorun
başlangıçta çözülmüş kabul edildi. Bu kontrol yalnızca HWND'nin görünürlük
bayrağını ölçüyordu; ikinci monitörde DWM/Explorer tarafından gösterilmeye
devam eden son kompozit kareyi ölçmüyordu.

Detach sırasını değiştirmek, pencereyi ekran dışına taşımak, dekorasyon
bitlerini kaldırmak ve `RedrawWindow` ile Explorer katmanlarını yenilemek de
tek başına kalıcı çözüm olmadı. Bazı denemeler çerçeveyi geçici olarak
temizledi ancak sonraki yeniden çizimde eski görüntü geri geldi.

### Kök neden

Wallpaper HWND'si Explorer'ın belgelenmemiş WorkerW masaüstü katmanına child
olarak bağlanıyordu. Kapatma sırasında pencerenin yeniden top-level hale
getirilmesi ve eski `GWL_STYLE` değerlerinin geri yüklenmesi, pencere gizli
olsa bile DWM/Explorer'ın son yüzeyi ve top-level çerçeveyi ikinci monitörün
kompozisyon önbelleğinde tutmasına neden oluyordu.

Bu nedenle hata, normal bir “pencere hâlâ görünür” problemi değil; WorkerW
reparent işlemi, top-level pencere stili ve Windows masaüstü kompozisyon
önbelleğinin birlikte oluşturduğu bir hayalet görüntü problemiydi. Negatif
monitör koordinatı ve farklı ekran yönü sorunun ikinci ekranda daha görünür
olmasını sağladı, fakat asıl neden koordinat hesabı değildi.

### Uygulanan çözüm

Wallpaper kapanırken gizli ve tekrar kullanılacak bir WebViewWindow bırakma
yaklaşımı kaldırıldı:

1. Yönetim penceresi önce görünür ve odaklanmış hale getiriliyor.
2. Wallpaper penceresinin WorkerW/etkileşim durumları Rust state'inden temizleniyor.
3. Tauri wallpaper `WebviewWindow` nesnesi ve native HWND tamamen yok ediliyor.
4. Windows'un `IDesktopWallpaper` API'siyle her monitörün mevcut duvar kâğıdı
   yolu okunup aynı monitöre yeniden uygulanıyor. Böylece kullanıcı ayarı
   değiştirilmeden Explorer/DWM masaüstü katmanı yeniden oluşturuluyor.
5. Wallpaper tekrar istendiğinde `WebviewWindowBuilder` ile aynı `wallpaper`
   etiketi altında dekorasyonsuz, görev çubuğunda görünmeyen yeni ve temiz bir
   pencere oluşturuluyor.

Yönetim penceresinin `CloseRequested` olayı ayrıca engellenerek `hide()`
çağrısına yönlendirildi. Böylece pencerenin X düğmesi uygulamayı sonlandırmıyor
veya görev çubuğuna küçültmüyor; uygulama sistem tepsisinde çalışmaya devam
ediyor.

### Değiştirilen temel noktalar

- `src-tauri/src/commands.rs`: Dinamik wallpaper oluşturma ve tamamen yok etme akışı.
- `src-tauri/src/desktop_host.rs`: Native state temizliği ve monitör bazlı masaüstü yenileme.
- `src-tauri/src/lib.rs`: Control X için sistem tepsisine gizleme ve wallpaper kapanış yönlendirmesi.
- `src-tauri/Cargo.toml`: Windows COM ve Shell API özellikleri.

### Doğrulama

- Hata doğrudan ikinci monitörde yeniden üretildi; yalnızca
  `IsWindowVisible` sonucuna güvenilmedi.
- Kapanıştan sonra ikinci monitörün gerçek ekran görüntüsü alınıp görsel olarak
  kontrol edildi.
- Yok edilen wallpaper penceresi yeniden oluşturuldu ve iki ardışık
  aç–kapat döngüsü uygulandı.
- Her iki döngüde de siyah pencere, başlık çubuğu veya beyaz çerçeve kalmadı.
- Uygulama süreci çalışmaya ve yanıt vermeye devam etti.
- `cargo check` başarılı oldu ve 7 Rust birim testinin tamamı geçti.

### Çıkarılan dersler

- WorkerW ve DWM problemlerinde HWND görünürlük bayrağı, ekrandaki gerçek
  sonucu kanıtlamaz.
- Çoklu monitör hataları hedef monitörün gerçek fiziksel koordinatlarında ve
  görüntü yakalama ile doğrulanmalıdır.
- Belgelenmemiş Explorer katmanlarına bağlanan pencerelerde gizleme ve detach
  işlemi yeterli olmayabilir; pencere yaşam döngüsünü sonlandırmak daha güvenli
  olabilir.
- Masaüstünü zorla yenilerken global wallpaper ayarını değiştiren eski Win32
  çağrıları yerine monitör başına mevcut değeri koruyan `IDesktopWallpaper`
  kullanılmalıdır.

## FD-WIN-002 — Explorer yeniden başlatıldığında WorkerW bağlantısının kaybı

- Tarih: 15 Temmuz 2026
- Durum: Çözüldü ve gerçek Explorer yeniden başlatmasıyla doğrulandı
- Final rapora dahil et: Evet

Explorer kapandığında yalnızca WorkerW handle'ı değil, onun child'ı olan Tauri
wallpaper HWND'si de Windows tarafından yok edildi. Tauri 2, dışarıdan yok
edilen HWND için pencere etiketini public API üzerinden temizlemediğinden aynı
süreç içinde aynı `wallpaper` etiketiyle güvenilir bir WebView oluşturulamadı.

Üç saniyelik watchdog yalnızca WorkerW modunun aktif olması gerektiği durumda
native pencere ve parent handle'larını doğrular. Bağlantı geçersizleşirse app
data dizinine tek kullanımlık bir kurtarma işareti yazılır ve interactivebackground Tauri'nin
kontrollü `request_restart` akışıyla yeniden başlatılır. Yeni süreç
`RunEvent::Ready` olayında işareti görür, SQLite ayarlarını yükler, wallpaper'ı
yeniden oluşturur ve yeni Explorer sürecinin WorkerW katmanına bağlar. İşaret
yalnızca başarılı kurtarma sonrasında silinir.

Gerçek testte Explorer PID'si ve uygulama PID'si değişti, uygulama yanıt vermeye
devam etti ve yeni wallpaper HWND'sinin parent sınıfı native olarak `WorkerW`
şeklinde doğrulandı. Normal kullanıcı kapatmasında kurtarma isteği oluşmadığı da
ayrıca kontrol edildi.

## ADR-001 — Ürün adının interactivebackground olarak değiştirilmesi

- Tarih: 15 Temmuz 2026
- Durum: Uygulandı
- Final rapora dahil et: Evet

Görünen ürün adı, pencere başlıkları, tray metinleri, npm paketi, Rust package ve
crate adları ile Windows executable adı `interactivebackground` olarak
değiştirildi. Önceki geliştirme verilerinin kaybolmaması için Tauri identifier
`com.flowdesk.app` ve SQLite dosya adı `flowdesk.db` geçici olarak legacy veri
kimliği şeklinde korundu. Teknik olay kayıtlarındaki eski Flowdesk ifadeleri,
olayın yaşandığı sürümün gerçek pencere adını belgelemek için değiştirilmedi.

## FEATURE-001 — Aktiviteye göre otomatik sakin moda dönüş

- Tarih: 15 Temmuz 2026
- Durum: Uygulandı ve gerçek zamanlı test edildi
- Final rapora dahil et: Evet

Etkileşim modu için kapalı, 1, 5, 10 ve 15 dakika seçenekleri eklendi. Süre
SQLite `app_settings.auto_calm_minutes` alanında kalıcı tutulur. Eski
veritabanları açılırken sütun otomatik eklenir ve varsayılan değer 5 dakikadır.
Wallpaper WebView'indeki pointer ve klavye aktivitesi 15 saniye throttle ile
Rust state'ine iletilir. Watchdog süreyi saniyede bir kontrol eder; süre dolunca
`edit_mode` veritabanında kapatılır, UI event ile güncellenir ve native pencere
WorkerW sakin moduna geri bağlanır.

Gerçek testte süre geçici olarak 1 dakikaya ayarlandı. 60 saniye sonunda SQLite
`edit_mode=0`, logda otomatik geçiş kaydı ve wallpaper HWND parent sınıfı
`WorkerW` olarak doğrulandı. Test sonunda kullanıcı ayarı 5 dakikaya geri
yüklendi.

## FEATURE-002 — Windows ile birlikte tray'e gizli başlatma

- Tarih: 15 Temmuz 2026
- Durum: Uygulandı ve Windows kayıt defteriyle doğrulandı
- Final rapora dahil et: Evet

Resmî Tauri 2 autostart eklentisi eklendi. Yönetim ekranındaki anahtar sistemin
gerçek autostart durumunu `isEnabled` ile okur; `enable` ve `disable` işlemleri
başarısız olursa UI eski duruma geri döner ve hata kullanıcıya gösterilir.
Gerekli capability izni yalnızca mevcut `control` ve `wallpaper` pencerelerini
kapsayan ana capability'ye eklendi.

Başlangıç kaydı `interactivebackground.exe --hidden` komutunu kullanır.
`--hidden` argümanı setup sırasında yönetim penceresini gizler fakat tray,
global kısayol, Explorer watchdog ve diğer servisleri açık bırakır. Manuel
başlangıç davranışı değişmez.

Gerçek testte UI anahtarı açıldığında HKCU Run kaydı ve `--hidden` argümanı
doğrulandı; anahtar kapatılınca kayıt tamamen silindi. Binary doğrudan
`--hidden` ile çalıştırıldığında control ve wallpaper native pencerelerinin
görünmez, sürecin ise yanıt verir durumda olduğu kontrol edildi. Test sonunda
autostart kapalı bırakıldı.

## FEATURE-003 — Windows kurulum paketleri

- Tarih: 15 Temmuz 2026
- Durum: Uygulandı ve gerçek release paketleri üretildi
- Final rapora dahil et: Evet

Dağıtım hedefleri NSIS ve WiX olarak açıkça sabitlendi. NSIS paketi yönetici
izni gerektirmeyen `currentUser` modunda kurulur ve kurulum başında Türkçe veya
İngilizce dil seçimi sunar. WiX tarafı aynı sürüm için Türkçe ve İngilizce MSI
paketleri üretir. Ürün adı ileride değişse bile Windows'un uygulamayı ayrı bir
ürün sanmaması için WiX upgrade code
`87d06055-f5ac-5cc7-8fc3-fd9d28902c89` olarak kalıcılaştırıldı.

WebView2 bulunmayan sistemlerde küçük Microsoft bootstrapper'ı sessizce
indirip kuran mod seçildi. Daha yeni bir sürüm kuruluysa eski paketin üzerine
downgrade yapılması engellendi. Paketler henüz ticari bir kod imzalama
sertifikasıyla imzalanmadığı için ilk yayınlarda Windows bilinmeyen yayıncı
uyarısı gösterebilir.

Release doğrulamasında optimize edilmiş `interactivebackground.exe`, 2,70 MB
NSIS setup ve 3,95 MB boyutunda iki MSI başarıyla üretildi. Paketlerin SHA-256
özetleri alındı ve Windows metadata'sında ürün/sürüm değerleri
`interactivebackground 0.1.0` olarak doğrulandı. Tauri paketleyicisinin
`__TAURI_BUNDLE_TYPE` işaretiyle ilgili verdiği uyarı mevcut kurulumları
etkilemez; henüz eklenmemiş updater özelliği devreye alınmadan önce Tauri CLI
ve crate sürümleriyle birlikte yeniden değerlendirilecektir.

## FEATURE-004 — Uygulama ikonu ve görsel kimlik

- Tarih: 15 Temmuz 2026
- Durum: Uygulandı
- Final rapora dahil et: Evet

Marka işareti, wallpaper manzarasını ve üst üste dizilmiş görev kartlarını tek
bir sembolde birleştirecek şekilde üretildi. Arayüzün orman yeşili, fildişi,
adaçayı ve altın renkleri kullanıldı; küçük görev çubuğu boyutlarında okunması
için metin, ince çizgi ve küçük detaylardan kaçınıldı.

Kaynak görsel şeffaf köşeli PNG olarak projede saklandı ve Tauri CLI ile ICO,
ICNS, Windows Store ile 32/128/256 piksel uygulama varlıklarına dönüştürüldü.
Aynı ikon yönetim penceresi başlığında ve wallpaper marka etiketinde
kullanılarak executable, installer ve uygulama içi kimlik tekleştirildi.

İlk yeşil–fildişi görev/manzara işareti kullanıcı değerlendirmesinde fazla
jenerik ve mevcut renk paleti fazla kurumsal bulunduğu için final marka olarak
kullanılmadı. Grok, Firefox, Spotify ve Things ikonlarının ortak güçlü yönleri
olan belirgin siluet, hareket, küçük boyutta okunabilirlik ve özenli uygulama
ikonu hissi incelendi; hiçbir markanın şekli kopyalanmadan alternatifler
üretildi.

Finalde `Folded Horizon` adı verilen özgün katlanmış şerit seçildi. Siyah zemin,
kırık beyaz dış şerit, lacivert iç kıvrım ve mat mercan vurgu kullanıldı. Final
üretim geçişinde alt kıvrım kalınlaştırıldı, renk kontrastı artırıldı ve güçlü
parlaklıklar azaltıldı. 32×32 görev çubuğu çıktısı ayrıca görsel olarak kontrol
edildi. Seçilen kaynak Tauri ikonlarının, uygulama içi marka görselinin ve
installer varlıklarının tamamında eski tasarımın yerini aldı.

## FEATURE-005 — Kalıcı sistem/açık/koyu tema desteği

- Tarih: 15 Temmuz 2026
- Durum: Uygulandı ve görsel olarak doğrulandı
- Final rapora dahil et: Evet

`AppSettings` modeline `system`, `light` ve `dark` değerlerini taşıyan tema
tercihi eklendi. Değer SQLite `app_settings.theme_mode` sütununda tutulur; eski
veritabanları açılırken sütun varsayılan `system` değeriyle otomatik eklenir.
Mevcut `settings-changed` olayı sayesinde yönetim ve wallpaper pencereleri aynı
tercihi yeniden başlatma gerektirmeden uygular.

Frontend ilk frame'de işletim sisteminin `prefers-color-scheme` değerini
uygulayarak tema parlamasını önler. Kalıcı tercih yüklendikten sonra özel açık
veya koyu tema etkinleştirilir; `system` seçiliyken işletim sistemi değişiklik
olayı canlı dinlenir. Tema tercihi, çözümlenen görünüm ve `color-scheme`
değerleri document kökünde birlikte tutulur.

Arayüz renkleri ortak CSS token'larına taşındı. Açık görünüm kırık beyaz,
grafit ve gece laciverti; koyu görünüm ise koyu grafit, lacivert vurgu ve mat
mercan ilerleme rengi kullanır. Yönetim yüzeyleri, formlar, kartlar, seçim
alanları, wallpaper widget'ı ve düzenleme kontrolleri iki tema için ayrı
kontrast değerleri alır. `prefers-reduced-motion` tercihi de geçişleri devre
dışı bırakır.

Legacy tablo migration testi varsayılan sistem temasını, ayar testi koyu tema
yazma/okumayı ve veritabanını yeniden açma testi kalıcılığı doğrular. Açık,
koyu ve sistem görünümleri yerel arayüzde görsel olarak kontrol edildi; tema
seçicinin değerleri, hesaplanan CSS renkleri ve tarayıcı hata kayıtları ayrıca
doğrulandı. Release binary `--hidden` ile gerçek kullanıcı veritabanında
çalıştırıldı; `theme_mode` sütununun `system` varsayılanıyla eklendiği ve mevcut
ayarların değişmeden kaldığı salt okunur SQLite sorgusuyla teyit edildi.

## FEATURE-006 — Kalıcı Türkçe/İngilizce yerelleştirme

- Tarih: 15 Temmuz 2026
- Durum: Uygulandı ve uçtan uca doğrulandı
- Final rapora dahil et: Evet

Arayüzdeki sabit metinler bileşenlerden çıkarılarak tür kontrollü anahtar
tabanlı dil kaynaklarına taşındı. İlk paket Türkçe ve İngilizcedir; Türkçe
kaynağı İngilizce anahtar kümesini TypeScript seviyesinde eksiksiz uygulamak
zorundadır. Eksik veya fazla anahtar bu nedenle üretim derlemesini durdurur.

`AppSettings` modeline `system`, `tr` ve `en` değerlerini taşıyan dil tercihi
eklendi. Tercih SQLite `app_settings.language` sütununda saklanır; eski
veritabanları açılırken sütun veri kaybı olmadan `system` varsayılanıyla eklenir.
Mevcut `settings-changed` olayı yönetim ile wallpaper penceresini yeniden
başlatma olmadan eşitler. Sistem modu web tarafında `navigator.language` ve
`languagechange` olayını, Windows tray tarafında ise kullanıcının gerçek
Windows locale değerini kullanır.

Kısa ve uzun tarih gösterimleri `Intl.DateTimeFormat` ile seçili locale göre
üretilir. Dinamik görev adları yerelleştirilmez; kullanıcının yazdığı veri
aynen korunur. Rust katmanından gelen kullanıcıya açık hatalar frontend'de
kararlı kategorilere çevrilerek seçili dilde gösterilir; geliştirici günlükleri
özgün teknik ayrıntıyı korur.

Tray menüsü başlangıçta seçili dille oluşturulur ve ayar değiştirildiğinde
native menü uygulamayı yeniden başlatmadan yeniden kurulur. Türkçe/İngilizce
tray etiketleri Rust birim testiyle, dil kaynaklarının anahtar eşitliği
TypeScript derlemesiyle ve SQLite migration/kalıcılık davranışı depo testleriyle
doğrulanır.

Release binary `--hidden` ile gerçek kullanıcı veritabanında çalıştırıldı.
`language` sütununun `system` varsayılanıyla eklendiği; mevcut şablon, saydamlık,
düzenleme modu, monitör, sakin mod ve tema değerlerinin değişmeden kaldığı salt
okunur sorguyla teyit edildi. Aynı release derlemesinden NSIS ile Türkçe ve
İngilizce MSI paketleri de başarıyla üretildi.

## FEATURE-007 — Kullanıcı arka planları ve hazır tema koleksiyonu

- Tarih: 15 Temmuz 2026
- Durum: Uygulandı, görsel ve release düzeyinde doğrulandı
- Final rapora dahil et: Evet

Arka plan ayarları global `app_settings` satırına eklenmek yerine
`monitor_backgrounds` tablosunda monitör anahtarıyla tutuldu. Kaynak türü,
hazır tema, özel dosya yolu, görüntü yerleşimi, karartma ve blur değeri her
monitör için bağımsızdır. Hedef monitör değiştirilince ilgili profil yüklenir;
henüz profili olmayan monitör güvenli `Folded Horizon` varsayılanını alır.

İlk hazır koleksiyon `Folded Horizon`, `Midnight`, `Graphite` ve `Ember`
temalarından oluşur. Temalar harici bitmap veya lisanslı içerik kullanmaz;
uygulamanın marka paletinden türetilmiş CSS katmanlarıyla üretilir. Önizleme ve
gerçek wallpaper aynı `WallpaperSurface` bileşenini kullandığı için görünüm,
karartma ve bulanıklık iki pencerede birebir aynıdır.

Kullanıcı görseli resmi Tauri dosya seçiciyle alınır. JPG/JPEG, PNG ve WebP
dışındaki uzantılar; dosya imzası uzantıyla eşleşmeyen içerikler; boş veya
50 MB'tan büyük dosyalar Rust tarafında reddedilir. Geçerli görsel dış konumdan
doğrudan sunulmaz: atomik geçişle uygulamanın `backgrounds` veri klasörüne
kopyalanır. Böylece dialog tarafından verilen geçici scope yeniden başlatmada
kaybolsa veya kaynak dosya sonradan silinse bile wallpaper çalışmaya devam eder.

WebView yalnızca uygulama veri klasörünü okuyabilen Tauri asset protokolünü
kullanır. Ayar komutu özel dosya yolunun bu yönetilen klasör içinde olduğunu
canonical path kontrolüyle yeniden doğrular. Yeni görsel veya hazır tema
seçilince önceki yönetilen kopya temizlenir; kayıtlı dosya erişilemez hale
gelirse sistem hazır temaya güvenli biçimde döner.

Arayüzde hazır tema kartları, native görsel seçici, `Kapla`/`Sığdır`/`Uzat`
yerleşimi ve canlı karartma/blur kontrolleri Türkçe ve İngilizce eklendi. Dört
tema tarayıcıda tek tek değiştirilerek, 1280 piksel görünümde yatay taşma
olmadan ve iki dilde doğrulandı. Rust testleri monitör profillerinin ayrılığını,
dosya imzası reddini ve yönetilen klasöre eksiksiz kopyalamayı kapsar.

Release binary gerçek kullanıcı veritabanında `--hidden` çalıştırıldı;
`monitor_backgrounds` tablosunun yedi beklenen sütunla oluştuğu, mevcut görev ve
uygulama ayarlarının değişmediği salt okunur sorguyla doğrulandı. NSIS ile iki
MSI paketi asset protocol ve dialog eklentileriyle başarıyla üretildi.

## FEATURE-008 — Sınırlandırılmış widget yerleşim motoru

- Tarih: 15 Temmuz 2026
- Durum: Uygulandı ve sınır testleriyle doğrulandı
- Final rapora dahil et: Evet

Görev alanının konumu global ayarlara eklenmedi; `widget_layouts` tablosunda
monitör anahtarı ile `focus`/`kanban` şablonundan oluşan birleşik anahtarla
tutuldu. Konum ve boyut değerleri 0–1 aralığında normalize edildi. Böylece
fiziksel piksel ölçüsü veya DPI değişse bile widget monitörde aynı göreli alanı
kaplar; henüz kaydı olmayan ya da sonradan yeniden bağlanan monitör güvenli
şablon varsayılanını alır.

Yerleşim yalnızca düzenleme modunda değiştirilebilir. Başlık alanı sürükleme
tutamağı, kenar ve köşelerdeki sekiz görünmez tutamak yeniden boyutlandırma
noktasıdır. Pointer capture, imleç widget dışına çıksa bile etkileşimi kontrollü
bitirir. Hareket sırasında React yalnızca yerel canlı yerleşimi günceller;
pointer bırakıldığında tek SQLite yazımı yapılır. Böylece yüksek frekanslı fare
olayları veritabanına taşınmaz.

Motor yüzde 1,5 görünür kenar payı, çözünürlüğe göre hesaplanan en az 280×250
piksel eşdeğeri, yüzde 78 maksimum genişlik/yükseklik, isteğe bağlı yüzde 2,5
grid ve 12 piksel kenara yapışma uygular. Son koordinatlar tekrar sınırlandırılıp
yuvarlanır. Aynı minimum/maksimum ve görünür alan kuralları Rust katmanında da
doğrulanır; frontend atlatılsa bile geçersiz veya sonlu olmayan değer SQLite'a
yazılamaz. İlk uygulamada arayüzün yüzde 97'ye kadar büyümeye izin verip Rust'ın
daha erken reddedebilmesi ihtimali bulundu; maksimum yüzde 78 kuralı iki
katmanda ortaklaştırılarak kaydetme sonrası geri sıçrama engellendi.

Kilit açıkken sürükleme ve yeniden boyutlandırma tutamakları devre dışıdır.
Grid tercihi, kilit durumu, konum ve boyut birlikte saklanır; tek komutla ilgili
monitör/şablon kaydı silinip varsayılana dönülür. `widget-layout-changed` olayı
yönetim ve wallpaper pencerelerinin aynı yerleşimi yeniden okumasını sağlar.
Rust testleri monitör ve şablon ayrımını, sıfırlamayı, ekran dışı koordinatları
ve maksimum boyut reddini kapsar; TypeScript üretim derlemesi pointer motoru ve
iki dildeki kontrol yüzeyini doğrular.

Release binary `--hidden` ile gerçek kullanıcı veritabanında çalıştırıldı.
Sekiz beklenen sütuna sahip `widget_layouts` tablosunun oluştuğu, ilk açılışta
gereksiz satır yazılmadığı, mevcut beş görevin ve uygulama ayarlarının aynen
korunduğu salt okunur SQLite sorgusuyla teyit edildi. Aynı derlemeden NSIS ile
Türkçe ve İngilizce MSI paketleri başarıyla üretildi.
