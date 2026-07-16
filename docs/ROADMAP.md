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

Durum: Tamamlandı — 15 Temmuz 2026

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

Durum: Tamamlandı — 16 Temmuz 2026

- Günün şiiri: yalnızca kamu malı, açık lisanslı veya izinli içerik koleksiyonu.
- Ayet widget'ı: sure/ayet numarası, çeviri adı ve kaynak gösterimi zorunlu.
- Hadis widget'ı: eser, bölüm/numara ve doğrulanabilir kaynak gösterimi zorunlu.
- Dini içeriklerin tamamen isteğe bağlı olması; katalogdan eklenmedikçe
  masaüstünde gösterilmemesi ve tek tıkla kaldırılabilmesi.
- İlk sürümde ağ/API bağımlılığı yerine denetlenmiş çevrimdışı içerik paketi ve
  yerel takvim gününe bağlı deterministik yenileme politikası.
- Dil ve bölgeye göre uygun içerik; kaynağı belirsiz rastgele metin gösterilmez.

Kabul ölçütü: Her içerik kartı kaynağını açıkça gösterir, internet kesilince son
doğrulanmış içerik korunur ve kullanıcı özelliği tamamen kapatabilir.

## Faz 7 — İlk kullanım ve kişiselleştirme akışı

Durum: Tamamlandı — 16 Temmuz 2026

- İlk açılışta dil ve tema seçimi.
- Hedef monitör, arka plan, widget alanı ve global kısayol tanıtımı.
- Autostart seçeneğinin açık rızayla sunulması; varsayılan olarak kapalı kalması.
- Örnek widget yerleşimi ve tek tıkla boş başlangıç seçeneği.
- Karşılama akışını ayarlardan tekrar açabilme.

## Faz 8 — Yayın ve güncelleme

Durum: Altyapı tamamlandı — 16 Temmuz 2026. İlk production yayını için updater
anahtarlarının ve isteğe bağlı Windows yayıncı sertifikasının repository
ayarlarına eklenmesi bekleniyor.

- Tauri updater entegrasyonu ve imzalı güncelleme manifesti.
- Windows kod imzalama sertifikası.
- GitHub Actions ile test, paketleme, checksum ve GitHub Release otomasyonu.
- Legacy `com.flowdesk.app` identifier ve `flowdesk.db` için kontrollü veri
  taşıma planı.

Kabul ölçütü: Yeni kimlikle ilk açılış mevcut görev, widget, ayar ve yönetilen
arka planları kayıpsız taşır; eski kopyayı korur. Elle tetiklenen release akışı
testleri çalıştırır, imzalı updater artifact'ı, `latest.json`, NSIS/MSI paketleri
ve SHA-256 checksum dosyası üretir.

## Faz 9 — Stabilizasyon, yerleşim motoru ve ürün sadeleştirmesi

Durum: Planlandı — kullanıcı kabul testi geri bildirimleri işlendi.

Bu faz yeni özellik sayısını artırmadan önce mevcut masaüstü deneyimini güvenilir,
ölçekli ve anlaşılır hale getirir. Aşağıdaki sıra bağımlılık sırasıdır; görsel
yenileme, hatalı pencere yaşam döngüsünün ve ortak yerleşim geometrisinin üzerine
kurulmazsa aynı sorunları farklı biçimde tekrar üretir.

### 9.1 — Pencere yaşam döngüsü ve kritik dönüş hatası

Durum: Tamamlandı — 16 Temmuz 2026

- Wallpaper üzerindeki `Yönetim paneline dön` akışından sonra wallpaper'ın aynı
  süreç içinde tekrar oluşturulup WorkerW katmanına bağlanamaması düzeltilecek.
- `control` görünürlüğü ile wallpaper'ın varlığı/görünürlüğü ayrı durumlar olarak
  ele alınacak; buton etiketi eski React state'ine değil native durum makinesine
  dayanacak.
- Aç → yönetime dön → tekrar aç döngüsü en az 20 tekrar, ikinci monitör ve 4K
  monitör senaryolarıyla regresyon testine alınacak.
- Pencere yok etme, yeniden oluşturma, event sırası ve yönetim penceresinin X ile
  tray'e gizlenmesi tek bir yaşam döngüsü sözleşmesine bağlanacak.

Kabul ölçütü: Yönetim ve wallpaper arasında sınırsız geçiş yapılabilir; süreç
kapanmaz, siyah pencere kalmaz ve hedef monitör değişmez.

### 9.2 — Ortak ölçekli preview ve yerleşim motoru v2

Durum: Tamamlandı — 16 Temmuz 2026

- Yönetim preview'su seçilen fiziksel monitörün gerçek en-boy oranını koruyan
  letterbox canvas kullanacak. Preview ile wallpaper aynı normalize koordinat,
  minimum boyut ve çarpışma kurallarını paylaşacak.
- Preview içindeki widget'lar doğrudan sürüklenip yeniden boyutlandırılabilecek;
  yapılan değişiklik gerçek wallpaper'a anında yansıyacak.
- Sabit yüzde 2,5 grid yerine varsayılan yüzde 0,5–1 aralığında ince grid
  kullanılacak. Grid yoğunluğu seçilebilir olacak; geçici serbest hareket için
  değiştirici tuş ve hassas klavye nudge desteği eklenecek.
- Widget minimum boyutları ve başlangıç ölçüleri küçültülecek. Tür başına ölçü
  kuralları preview pikseline değil hedef monitör eşdeğerine göre hesaplanacak.
- Widget'ların birbirini ezmesi engellenecek. Geçersiz alan kırmızı hayaletle
  gösterilecek; bırakmada otomatik ve şaşırtıcı zincirleme itme yerine en yakın
  boş konum önerilecek veya son geçerli konuma dönülecek.
- Aynı çarpışma ve sınır doğrulaması Rust katmanında da uygulanacak; geçersiz
  yerleşim frontend atlatılarak SQLite'a yazılamayacak.

Kabul ölçütü: Preview ile gerçek ekranın yerleşimi aynı görünür; farklı DPI ve
en-boy oranlarında widget'lar çakışmaz, görünür alanın dışına çıkmaz ve kullanıcı
ince grid üzerinde hassas yerleşim yapabilir.

Uygulama notu: Preview, seçili monitörün fiziksel en-boy oranını koruyan ve DPI
ölçeğinden türetilen efektif WebView yüzeyinde render edilip tek katsayıyla
letterbox alanına küçültülür; böylece kart içeriği ve tipografi de geometriyle
aynı oranda ölçeklenir. Yüzde 0,5 ve yüzde 1 grid, Alt ile geçici
serbest hareket ve ok tuşlarıyla hassas nudge ortak hesaplayıcıyı kullanır.
Çakışan sürükleme kırmızı hayalet olarak gösterilir ve son geçerli konuma döner.
Rust/SQLite katmanı aynı çakışma, ekran sınırı ve hedef monitöre göre minimum
efektif boyut kurallarını tekrar doğrular; yeni ve çoğaltılan widget en yakın boş
konuma yerleştirilir. Mevcut kullanıcı yerleşimleri migration ile zorla
değiştirilmez.

### 9.3 — Widget yoğunluğu ve düzenleme affordance'ları

Durum: Tamamlandı — 16 Temmuz 2026

- Kart padding, başlık yüksekliği, boş satırlar ve kontrol aralıkları azaltılacak;
  içerik yoğunluğu widget boyutuna responsive hale getirilecek.
- Düzenleme modunda her widget'ın üstünde ayrı bir sürükleme rayı/çizgisi ve kısa
  tutma ipucu gösterilecek. Normal modda bu yüzey tamamen kaybolacak.
- Resize kenarları görünür ama sakin tutamaklarla belirtilecek; kilitli durumun
  yalnızca renk değil ikon/metinle de anlaşılması sağlanacak.
- Etkileşim gerektiren widget'larda kontrollerin yalnızca etkileşim/düzenleme
  modunda çalıştığını belirten kısa alt bilgi gösterilecek. Devre dışı kontroller
  disabled semantiği ve açıklayıcı tooltip kullanacak.
- Mevcut `Focus` düğmesi işlevsiz dekoratif kontrol olduğu için kaldırılacak.
  Daha sonra Pomodoro başlatma veya tek görevi öne çıkarma davranışı açıkça
  tanımlanırsa yeni adı ve gerçek işleviyle geri eklenecek.

Kabul ölçütü: İlk kez kullanan biri yardım okumadan widget'ı nereden taşıyacağını,
nasıl boyutlandıracağını ve bir kontrolün neden pasif olduğunu anlayabilir.

Uygulama notu: Widget padding, başlık, görev satırı, Kanban kartı ve Pomodoro
kontrol aralıkları sıkılaştırıldı; yeni başlangıç Focus/Kanban yükseklikleri
azaltıldı. Container query kuralları dar ve kısa kartlarda ikincil içeriği
sadeleştiriyor. Düzenleme modunda bağımsız bir sürükleme rayı, sekiz görünür
resize tutamacı ve klavye taşıma açıklaması gösteriliyor. Kilitli widget rayında
ikon ve metinle işaretleniyor, resize yüzeyleri kaldırılıyor. Etkileşim gerektiren
kontroller sakin modda gerçek `disabled` semantiğine ve açıklayıcı alt bilgiye
sahip; işlevsiz Focus düğmesi kaldırıldı.

### 9.4 — Yönetim paneli bilgi mimarisi ve görsel düzen

- Sabit ve sıkışık iki kolon yerine görevler, masaüstü canvas'ı ve seçili widget
  ayarlarını ayıran responsive çalışma alanı kurulacak.
- Widget kataloğu sürekli buton kalabalığı olarak görünmeyecek; `Widget ekle`
  akışı ve seçili widget inspector'ı kullanılacak.
- Monitör, görünüm, arka plan ve davranış ayarları görev yönetiminden ayrılacak;
  birincil/ikincil eylem hiyerarşisi ortaklaştırılacak.
- Dar pencere ve 4K ölçeklemede canvas okunabilir kalacak; header ve kontrol
  paneli gereksiz alan tüketmeyecek.

Kabul ölçütü: Yönetim penceresinde yatay taşma veya üst üste binen kontrol
oluşmaz; ana eylemler tek bakışta bulunur ve seçili widget ekrandan ayrılmadan
düzenlenebilir.

**Tamamlandı.** Yönetim yüzeyi görev rayı, ölçeklenen canlı canvas ve kalıcı
inspector olarak üçe ayrıldı. Widget kataloğu açılır `Widget ekle` akışına
taşındı; ekrandaki widget'lar yatay seçicide listeleniyor ve görünürlük, kilit,
ızgara, sıra, çoğaltma, silme ile Pomodoro süreleri seçili widget kartından
yönetiliyor. Arka plan ve genel çalışma alanı ayarları ayrı sekmelere alındı.
1440×900 ve 820×900 tarayıcı regresyonunda yatay taşma oluşmadığı, dar görünümde
üç alanın okunabilir sırayla kaydırıldığı ve katalog/sekme akışlarının çalıştığı
doğrulandı.

### 9.5 — Pomodoro bildirim ve ses güvenilirliği

- Pomodoro tamamlanması uygulama açık, tray'de gizli ve wallpaper yeniden
  oluşturulmuş durumlarda native bildirimle test edilecek.
- Uygulama çalışırken paketlenmiş kısa bir tamamlanma sesi çalınacak; ses aç/kapat,
  seviye ve `Sesi dene` ayarları sunulacak.
- İşletim sistemi bildirim izni ilk ihtiyaçta açıklanarak istenecek; izin kapalıysa
  kullanıcıya ayarlara yönlendiren durum gösterilecek.
- Sistem sessiz modu veya Rahatsız Etmeyin tercihi uygulama tarafından aşılmayacak;
  bu durumda görsel tamamlanma durumu yine korunacak.

Kabul ölçütü: İzin verilen normal sistem koşullarında seans bitişi tek bildirim
ve tek ses üretir; duraklatılan/sıfırlanan sayaç eski bildirimi tetiklemez.

### 9.6 — Çekirdek widget'lar ve güvenli Widget Store

- Varsayılan çekirdek katalog dört araçla sınırlandırılacak: Odak Görevleri,
  Kanban, Pomodoro ve Saat. Tarih, Saat widget'ının görünüm seçeneklerinden biri
  veya küçük eşlikçi görünümü olacak.
- Günün şiiri, ayeti ve hadisi isteğe bağlı paketler olarak Widget Store'a
  taşınacak. Mevcut kullanıcıların kurulu widget'ları migration sırasında
  `installed` kabul edilecek ve silinmeyecek.
- İlk Store sürümü internetten rastgele JavaScript/Rust kodu çalıştırmayacak.
  Uygulamayla imzalı gelen, manifesti ve izinleri bilinen modüller yerel olarak
  kurulup kaldırılacak. Uzak üçüncü taraf kodu; sandbox, imza, izin ve inceleme
  modeli tasarlanmadan desteklenmeyecek.
- Store kartlarında açıklama, kaynak, gerekli minimum alan, veri/ağ izinleri ve
  kaldırma işleminin etkisi açıkça gösterilecek.
- İsteğe bağlı gelecek widget backlog'u:
  - **LeetCode Daily:** Günlük sorunun başlığı, zorluk seviyesi, konu etiketleri
    ve soru bağlantısı. İlk sürümde `yapılmadı / çalışılıyor / yapıldı` durumu
    cihazda yerel tutulacak; hesap ilerlemesini otomatik eşitleme ancak kararlı
    ve izin verilen bir entegrasyon yolu doğrulanırsa eklenecek.
  - **GitHub:** Kullanıcının açıkça seçtiği dar kapsamla contribution özeti,
    atanmış issue/PR veya repository durumundan biri. Kişisel erişim anahtarı
    düz metin SQLite'a yazılmayacak; bağlantı kurulursa OAuth/device flow ve
    işletim sistemi güvenli credential deposu kullanılacak. İlk tasarımda hangi
    bilginin gerçekten günlük değer ürettiği kullanıcıyla seçilecek.
  - **English Flashcards:** Günlük küçük kart destesi, kelime–anlam–örnek cümle,
    çevir/aç/kapat ve kısa oyun akışı. Kart başarısı yerel tutulacak, aralıklı
    tekrar algoritmasıyla `yeni / öğreniliyor / öğrenildi` durumlarına ayrılacak;
    temel paket çevrimdışı çalışacak.
- Ağ kullanan Store widget'ları cache, açık yenileme zamanı, hata fallback'i ve
  minimum izin ilkelerine uyacak; internet yokluğu masaüstü yüzeyini bozmayacak.

Kabul ölçütü: Yeni kullanıcı yalnızca sade çekirdek kataloğu görür; isteğe bağlı
widget yükleme/kaldırma veri kaybı veya uygulama yeniden başlatması gerektirmez.

### 9.7 — Saat widget'ı v2

- Dijital ve analog görünüm seçeneği.
- `Sistem`, 12 saat ve 24 saat formatı.
- Sistem saat dilimi veya IANA saat dilimi seçimi; seçilen bölgenin kısa adı.
- İsteğe bağlı saniye, tarih ve gün gösterimi.
- Widget'a özel ayarların genel tabloyu sürekli değiştirmeden genişleyebilmesi
  için sürümlü `settings_json`/typed settings modeli ve migration.

Kabul ölçütü: Birden fazla saat widget'ı farklı saat dilimi ve formatlarla aynı
anda doğru çalışır; yeniden başlatmada ayarlar korunur.

### 9.8 — Wallpaper koleksiyonu ve masaüstü temizliği

- Gerçek wallpaper yüzeyinden uygulama logosu, marka adı, `Projects`, `Recycle
  Bin` ve bütün dummy masaüstü öğeleri kaldırılacak. Düzenleme kılavuzları yalnızca
  edit modunda gösterilecek.
- En az dört koyu ve dört açık modern tema hazırlanacak; nötr, lacivert, grafit,
  kırık beyaz ve sınırlı vurgu renkleri kullanılacak.
- Tema preview'su ile gerçek wallpaper aynı asset/render yolunu kullanacak;
  yalnızca küçük CSS taklidi gösterilmeyecek.
- Açık ve koyu arka planlarda widget kontrastı otomatik okunabilirlik kontrolünden
  geçecek; kullanıcı görseli için karartma/blur ayarları korunacak.

Kabul ölçütü: Wallpaper üzerinde ürüne ait dummy içerik kalmaz; açık ve koyu
tema ailesi 16:9, ultrawide ve 4K ekranlarda bozulmadan görünür.

### 9.9 — Regresyon, performans ve beta kabul turu

- Pencere yaşam döngüsü, çarpışma, grid, preview dönüşümü, saat ayarları ve
  Pomodoro tamamlanması için Rust/TypeScript testleri.
- 1080p, 1440p, 4K, farklı DPI, negatif monitör koordinatı ve ikinci monitör
  senaryoları.
- Mevcut kullanıcı widget'ları, özel arka planları ve ayarları için migration
  snapshot testi.
- Temiz `v0.2.0-beta` paketi üzerinde kullanıcı kabul turu; kritik olmayan yeni
  özellikler bu tur tamamlanana kadar bekletilecek.

Kabul ölçütü: Bildirilen kritik akışlar tekrar üretilemez, mevcut veriler
korunur ve beta checklist'in bütün kritik maddeleri geçer.

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

Faz 9.5 ile devam edilmeli; Pomodoro tamamlanma akışı native bildirim, paketlenmiş
ses ve izin/durum geri bildirimiyle uygulama yaşam döngüsü boyunca güvenilir hale
getirilmelidir.
