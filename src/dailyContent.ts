import type { LanguagePreference, WidgetKind } from "./types";

export type DailyContent = {
  id: string;
  text: string;
  original?: string;
  attribution: string;
  reference?: string;
  sourceUrl: string;
  originalSourceUrl?: string;
  license: string;
  note?: string;
};

type LocalizedText = { tr: string; en: string };

type ContentEntry = {
  id: string;
  text: LocalizedText;
  original?: string;
  attribution: LocalizedText;
  reference?: LocalizedText;
  sourceUrl: LocalizedText;
  originalSourceUrl?: string;
  license: LocalizedText;
  note?: LocalizedText;
};

const poems: ContentEntry[] = [
  {
    id: "yunus-ilim",
    text: {
      tr: "İlim ilim bilmekdir\nİlim kendin bilmekdir\nSen kendini bilmezsin\nYa nice okumakdır",
      en: "“Hope” is the thing with feathers —\nThat perches in the soul —",
    },
    attribution: { tr: "Yunus Emre", en: "Emily Dickinson" },
    reference: { tr: "İlim İlim Bilmektir", en: "Hope is the thing with feathers" },
    sourceUrl: {
      tr: "https://tr.wikisource.org/wiki/%C4%B0lim_%C4%B0lim_Bilmektir",
      en: "https://en.wikisource.org/wiki/Poems_(Dickinson,_1890)/Life/XXXII",
    },
    license: { tr: "Kamu malı metin", en: "Public-domain text" },
  },
  {
    id: "yunus-sevelim",
    text: {
      tr: "Gelin tanış olalım,\nİşin kolayın tutalım\nSevelim sevilelim,\nDünya kimseye kalmaz",
      en: "To see a World in a Grain of Sand\nAnd a Heaven in a Wild Flower",
    },
    attribution: { tr: "Yunus Emre", en: "William Blake" },
    reference: { tr: "Sevelim sevilelim", en: "Auguries of Innocence" },
    sourceUrl: {
      tr: "https://tr.wikisource.org/wiki/Sevelim_sevilelim",
      en: "https://en.wikisource.org/wiki/Auguries_of_Innocence",
    },
    license: { tr: "Kamu malı metin", en: "Public-domain text" },
  },
  {
    id: "fuzuli-rossetti",
    text: {
      tr: "Beni candan usandırdı\nCefâdan yâr usanmaz mı",
      en: "Who has seen the wind?\nNeither I nor you:\nBut when the leaves hang trembling,\nThe wind is passing through.",
    },
    attribution: { tr: "Fuzûlî", en: "Christina Rossetti" },
    reference: { tr: "Beni candan usandırdı", en: "Who Has Seen the Wind?" },
    sourceUrl: {
      tr: "https://tr.wikisource.org/wiki/Beni_Candan_Usand%C4%B1rd%C4%B1",
      en: "https://en.wikisource.org/wiki/Sing-Song:_A_Nursery_Rhyme_Book/Who_has_seen_the_wind%3F",
    },
    license: { tr: "Kamu malı metin", en: "Public-domain text" },
  },
];

const verses: ContentEntry[] = [
  {
    id: "quran-94-5",
    original: "فَإِنَّ مَعَ الْعُسْرِ يُسْرًا",
    text: { tr: "Demek ki zorlukla beraber bir kolaylık var.", en: "But lo! with hardship goeth ease," },
    attribution: { tr: "Kur’an-ı Kerim", en: "The Qur’an" },
    reference: { tr: "İnşirâh 94:5", en: "Ash-Sharh 94:5" },
    sourceUrl: {
      tr: "https://tr.wikisource.org/wiki/Kur%27an/%C4%B0n%C5%9Fir%C3%A2h_Suresi",
      en: "https://en.wikisource.org/wiki/Solace_%28Qur%27an%29",
    },
    originalSourceUrl: "https://tanzil.net/#94:5",
    license: { tr: "Arapça: Tanzil CC BY 3.0 · Meal: Elmalılı (kamu malı)", en: "Arabic: Tanzil CC BY 3.0 · Translation: Pickthall (public domain)" },
  },
  {
    id: "quran-20-114",
    original: "وَقُل رَّبِّ زِدْنِي عِلْمًا",
    text: { tr: "Rabbım artır beni ılimce.", en: "and say: My Lord! Increase me in knowledge." },
    attribution: { tr: "Kur’an-ı Kerim", en: "The Qur’an" },
    reference: { tr: "Tâhâ 20:114", en: "Ta-Ha 20:114" },
    sourceUrl: {
      tr: "https://tr.wikisource.org/wiki/Kur%27an/T%C3%A2h%C3%A2_Suresi",
      en: "https://en.wikisource.org/wiki/Ta-Ha",
    },
    originalSourceUrl: "https://tanzil.net/#20:114",
    license: { tr: "Arapça: Tanzil CC BY 3.0 · Meal: Elmalılı (kamu malı)", en: "Arabic: Tanzil CC BY 3.0 · Translation: Pickthall (public domain)" },
  },
  {
    id: "quran-112-1",
    original: "قُلْ هُوَ اللَّهُ أَحَدٌ",
    text: { tr: "De, o: Allah tek bir tektir.", en: "Say: He is Allah, the One!" },
    attribution: { tr: "Kur’an-ı Kerim", en: "The Qur’an" },
    reference: { tr: "İhlâs 112:1", en: "Al-Ikhlas 112:1" },
    sourceUrl: {
      tr: "https://tr.wikisource.org/wiki/Kur%27an/%C4%B0hl%C3%A2s_Suresi",
      en: "https://en.wikisource.org/wiki/The_Unity_%28Qur%27an%29",
    },
    originalSourceUrl: "https://tanzil.net/#112:1",
    license: { tr: "Arapça: Tanzil CC BY 3.0 · Meal: Elmalılı (kamu malı)", en: "Arabic: Tanzil CC BY 3.0 · Translation: Pickthall (public domain)" },
  },
];

const hadiths: ContentEntry[] = [
  hadith("intentions", "إِنَّمَا الأَعْمَالُ بِالنِّيَّاتِ", "Sahih al-Bukhari 1", "https://sunnah.com/bukhari:1"),
  hadith("counsel", "الدِّينُ النَّصِيحَةُ", "Sahih Muslim 55a", "https://sunnah.com/muslim:55a"),
  hadith("purity", "الطُّهُورُ شَطْرُ الإِيمَانِ", "Sahih Muslim 223", "https://sunnah.com/muslim:223"),
  hadith("kind-word", "الْكَلِمَةُ الطَّيِّبَةُ صَدَقَةٌ", "Sahih al-Bukhari 2989", "https://sunnah.com/bukhari:2989"),
  hadith("brother", "لاَ يُؤْمِنُ أَحَدُكُمْ حَتَّى يُحِبَّ لأَخِيهِ مَا يُحِبُّ لِنَفْسِهِ", "Sahih al-Bukhari 13", "https://sunnah.com/bukhari:13"),
];

export function getDailyContent(kind: Extract<WidgetKind, "dailyPoem" | "dailyVerse" | "dailyHadith">, language: LanguagePreference, date: Date): DailyContent {
  const locale = resolveLanguage(language);
  const entries = kind === "dailyPoem" ? poems : kind === "dailyVerse" ? verses : hadiths;
  const offsets = { dailyPoem: 0, dailyVerse: 17, dailyHadith: 31 } as const;
  const entry = entries[positiveModulo(localDayNumber(date) + offsets[kind], entries.length)];
  return {
    id: entry.id,
    text: entry.text[locale],
    original: entry.original,
    attribution: entry.attribution[locale],
    reference: entry.reference?.[locale],
    sourceUrl: entry.sourceUrl[locale],
    originalSourceUrl: entry.originalSourceUrl,
    license: entry.license[locale],
    note: entry.note?.[locale],
  };
}

function hadith(id: string, arabic: string, reference: string, sourceUrl: string): ContentEntry {
  return {
    id: `hadith-${id}`,
    text: { tr: arabic, en: arabic },
    attribution: { tr: "Hadis-i şerif", en: "Hadith" },
    reference: { tr: reference, en: reference },
    sourceUrl: { tr: sourceUrl, en: sourceUrl },
    license: { tr: "Klasik Arapça kaynak metni · kamu malı", en: "Classical Arabic source text · public domain" },
    note: {
      tr: "Çeviri ve bağlam için doğrulanmış kaynak sayfasını aç.",
      en: "Open the verified source page for translation and context.",
    },
  };
}

function resolveLanguage(language: LanguagePreference): keyof LocalizedText {
  if (language === "tr" || language === "en") return language;
  return navigator.language.toLocaleLowerCase().startsWith("tr") ? "tr" : "en";
}

function localDayNumber(date: Date) {
  return Math.floor(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()) / 86_400_000);
}

function positiveModulo(value: number, divisor: number) {
  return ((value % divisor) + divisor) % divisor;
}
