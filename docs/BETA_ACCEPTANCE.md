# v0.2.0 beta build 1 kabul checklist'i

Bu belge Faz 9.9 boyunca dondurulan beta kapsamını ve her kritik maddenin
doğrulama kanıtını tutar. Kritik olmayan yeni özellikler bu tur kapanana kadar
beta kapsamına alınmaz.

Paket sürümü `0.2.0-1` olarak tutulur. Sayısal prerelease kimliği Windows MSI
araç zincirinin kuralıdır; ürün açısından bu paket `v0.2.0-beta.1` adayıdır.

## Otomatik geçiş kapıları

- [x] TypeScript derleme ve Vite production build'i hatasız.
- [x] Frontend regresyonları: çarpışma, yüzde 1 grid, Alt ile serbest hareket,
  resize minimumları, preview/DPI dönüşümü ve saat ayarları.
- [x] Monitör matrisi: 1920×1080 @1x, 2560×1440 @1.25x,
  3840×2160 @2x ve negatif koordinatlı 3440×1440 ikinci monitör @1.5x.
- [x] Rust regresyonları: pencere yaşam döngüsü durum makinesi, native yerleşim
  doğrulaması, Pomodoro tek-sefer tamamlanması ve veri migration'ları.
- [x] Migration snapshot: görev, ayar, hedef monitör, özel arka plan yolu,
  widget konumu, analog saat ayarı ve Pomodoro durumu birlikte korunuyor.
- [x] Frontend production bundle bütçesi: JS en fazla 400 KiB, CSS en fazla
  80 KiB (minified, sıkıştırılmamış toplam).
- [x] Windows release NSIS ile Türkçe/İngilizce MSI paketleri temiz build'den
  üretildi.

## Kullanıcı kabul turu — kritik manuel kontroller

Bu bölüm gerçek Windows masaüstü/Explorer davranışı gerektirdiği için paket
kurulduktan sonra kullanıcıyla birlikte işaretlenir.

- [ ] Yönetim → wallpaper → yönetime dön döngüsü 20 kez çalışıyor; süreç
  kapanmıyor ve hedef monitör değişmiyor.
- [ ] İkinci/4K monitörde wallpaper doğru fiziksel sınırda; sağa kayma yok.
- [ ] Wallpaper kapatılınca ikinci monitörde siyah veya eski pencere karesi
  kalmıyor; X yönetim penceresini tray'e gizliyor.
- [ ] Preview'da yapılan sürükleme/resize gerçek wallpaper ile aynı konumu
  veriyor; widget'lar üst üste bırakılamıyor.
- [ ] Çalışan Pomodoro yönetim penceresi tray'deyken bir bildirim ve bir ses
  üretiyor; duraklatma/sıfırlama eski uyarıyı tetiklemiyor.
- [ ] Mevcut kullanıcı veritabanıyla yükseltmede görevler, özel arka plan,
  widget'lar ve ayarlar korunuyor.

## Tekrarlanabilir komutlar

```powershell
npm test
npm run build
npm run check:bundle
cargo test --manifest-path src-tauri\Cargo.toml
npx tauri build
```

Kritik manuel maddelerin tamamı işaretlenmeden `v0.2.0` kararlı sürüm etiketi
oluşturulmaz.
