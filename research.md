Fiqh Compliance and Architectural Validation Report: The zakatrs Library
1. Executive Summary: The Intersection of Divine Law and Computational Precision
This report constitutes a formal Sharia compliance audit and architectural validation of the zakatrs library, a Rust-based computational engine designed for the calculation of Zakat. As the primary author—functioning in the dual capacity of an expert Islamic Jurist (Faqih) specializing in Fiqh al-Zakah and a Senior Systems Architect—I present this analysis to certify that the library’s internal logic adheres strictly to the primary sources of Islamic Law (Quran and Sunnah), the consensus of the scholars (Ijma'), and valid analogical deduction (Qiyas).
The development of zakatrs represents a critical intersection between the immutable laws of the Sharia and the rigorous demands of modern software engineering. Zakat, being the third pillar of Islam, is not merely a charitable contribution but a precise act of worship (Ibadah) governed by specific thresholds (Nisab), holding periods (Hawl), and rates (Miqdar). Any deviation in calculation—whether due to floating-point errors in code or misinterpretation of jurisprudential texts—renders the act invalid and potentially sinful. Therefore, the architectural decision to utilize rust_decimal for fixed-point precision is not merely a technical optimization but a fulfillment of the religious obligation of Ihsan (excellence/perfection) and the avoidance of Gharar (ambiguity) in financial worship.
This report validates the library's "Pluggable Strategy Pattern" as the architectural embodiment of the valid divergence (Ikhtilaf) found in Islamic jurisprudence. By allowing the injection of distinct strategies (e.g., HanafiStrategy, ShafiStrategy), the library honors the rich intellectual tradition of the four Sunni Madhabs, permitting users to calculate Zakat according to the methodology they follow, particularly in contentious areas such as the taxation of jewelry and the aggregation of diverse assets.
The following sections provide an exhaustive Takhrij (extraction) of the legal proofs (Dalil) supporting every function within the codebase, referencing classical texts like Al-Majmu' and Al-Hidayah alongside contemporary resolutions from AAOIFI and the International Islamic Fiqh Academy (IIFA).
2. Precious Metals: The Gold and Silver Standards
Module: src/maal/precious_metals.rs
The calculation of Zakat on gold (Dhahab) and silver (Fiddah) serves as the foundational bedrock for all monetary Zakat. The precious_metals.rs module handles weight verification, purity adjustment, and the application of the exemption logic for permissible jewelry.
2.1 The Nisab: Establishing the Threshold of Wealth
Code Logic: The library verifies if the input weight equals or exceeds the configured Nisab. The default configuration sets these thresholds at 85 grams for gold and 595 grams for silver.
Fiqh Verification:
The legislative authority for these thresholds is derived directly from the statements of the Prophet Muhammad (ﷺ). The classical measurements were defined as 20 Dinars for gold and 200 Dirhams for silver.
Primary Evidence (Dalil):
The definitive Hadith establishing the gold and silver thresholds is narrated by ‘Ali bin Abi Talib (ra):"When you possess two hundred dirhams and one year passes on them, five dirhams are payable. Nothing is incumbent on you, that is, on gold, till it reaches twenty dinars. When you possess twenty dinars and one year passes on them, half a dinar is payable. Whatever exceeds, that will be reckoned properly."
— Sunan Abu Dawud (Hadith 1573) 1; Jami` at-Tirmidhi (Hadith 620).
Grading: Sahih (Authentic) by Al-Albani and Al-Bukhari.
Furthermore, regarding silver specifically, the Prophet (ﷺ) stated in the Muttafaq Alayh (agreed upon) narration:"There is no Zakat on less than five Awaq (of silver)."
— Sahih Bukhari (Hadith 1447) 2; Sahih Muslim (Hadith 979).3
Scholarly Conversion (Tahqiq al-Manat):
The conversion of these classical weights into modern metric units is a subject of scholarly refinement (Tahqiq).
Silver: The consensus is that 5 Awaq equals 200 Dirhams. A classical legal Dirham is determined to weigh approximately 2.975 grams.

$$200 \text{ Dirhams} \times 2.975 \text{ grams/Dirham} = 595 \text{ grams}.$$

The library’s default of 595g is strictly compliant with the majority view of modern jurists and standards bodies.4
Gold: The threshold is 20 Dinars. A legal Dinar weighs one Mithqal, which is equivalent to 4.25 grams.

$$20 \text{ Dinars} \times 4.25 \text{ grams/Dinar} = 85 \text{ grams}.$$

The library’s usage of 85g aligns with the standards set by the Organization of Islamic Cooperation (OIC) and AAOIFI Sharia Standard No. 35.5
2.2 Jewelry Exemption (Huliyy al-Mubah)
Code Logic: The PreciousMetals struct includes a usage enum (JewelryUsage::PersonalUse vs. Investment). The calculate_zakat function checks the ZakatStrategy to see if jewelry_exempt is true. If true, and the usage is personal, Zakat is waived.
Fiqh Analysis:
This logic correctly models the significant juristic divergence regarding Huliyy al-Mubah (permissible jewelry for women). The library allows the user to select their Madhab, thereby toggling the calculation logic to match their legal adherence.
The View of Exemption (Shafi'i, Maliki, Hanbali):
The majority of scholars hold that jewelry intended for personal, permissible ornamentation is exempt from Zakat. They categorize it under Qunyah (personal possession) rather than Nama' (growing wealth), analogizing it to clothing or working camels which are exempt.
Primary Evidence: The practice of the Mother of the Believers, Aisha (ra)."Aisha (ra) used to look after her brother's orphaned daughters in her apartment. They had jewelry, and she did not pay Zakat on their jewelry."
— Al-Muwatta Imam Malik (Book 17, Section: Zakat on Jewelry).7
Classical Citations:
Shafi'i: Imam Al-Nawawi states in Al-Majmu' Sharh al-Muhadhdhab: "If a person possesses jewelry... and intends it for permissible use... then the correct view of the Madhab is that no Zakat is obligatory on it".7
Hanbali: Ibn Qudamah in Al-Mughni confirms: "There is no Zakat on women's jewelry if it is worn or lent".8
Maliki: Documented in Al-Muwatta, confirming no Zakat on pearls, musk, or amber, extending to gold worn for adornment.10
The View of Obligation (Hanafi):
The Hanafi school maintains that Zakat is obligatory on gold and silver regardless of its form (bullion, coin, or jewelry). Their juristic reasoning (Ta'lil) is that gold and silver possess intrinsic value (Thamaniyyah) and potential for growth, which cannot be negated by crafting them into ornaments.
Primary Evidence: The Hadith of the bracelets of fire.A woman came to the Messenger of Allah (ﷺ) with her daughter, who wore two heavy gold bangles... He said: "Would you be pleased that Allah puts two bangles of fire on your hands?"
— Sunan Abu Dawud (Hadith 1558); Sunan An-Nasai.11
Classical Citation: Imam Al-Kasani (Hanafi) argues in Bada'i al-Sana'i regarding jewelry: "It becomes a blessing that enables one to enjoy luxury, obligating the expression of gratitude by allocating a portion of it to the poor".13 This is further reinforced in Al-Hidayah by Al-Marghinani, stating Zakat is due on gold/silver unconditionally.14
Architectural Validation: The ZakatStrategy trait is the perfect mechanism to handle this. By abstracting the rule jewelry_exempt, the library avoids imposing a single view on all users, ensuring broad Sharia compliance across schools.
2.3 Purity and Karat Conversion
Code Logic: The library normalizes the weight of gold to a 24K standard using the formula:


$$\text{Effective Weight} = \text{Total Weight} \times \left( \frac{\text{Karat}}{24} \right)$$
Fiqh Analysis:
Zakat is mandated on the pure metal content (Khalis), not on the alloys (copper, zinc, nickel) mixed to harden the jewelry.
Scholarly Consensus: If the gold is mixed with other substances, Zakat is due only if the pure gold content reaches the Nisab. This requires mathematically extracting the pure gold weight.
Contemporary Fatwa: "To calculate Zakat on gold, the number of grams is multiplied by the caratage, then divided by 24. The resulting figure represents the pure, 24 karat gold. If that number reaches 85 grams... Zakat must be given." — IslamQA Fatwa 214221.15
Validation: This calculation prevents Zulm (injustice) by ensuring the user does not pay Zakat on non-zakatable base metals found in 18K or 21K jewelry.
3. Business Assets: Trade Goods (Urud al-Tijarah)
Module: src/maal/business.rs
The business.rs module implements the logic for calculating Zakat on commercial entities, applying the formula: (Cash + Inventory + Receivables) - Current Liabilities.
3.1 The Obligation of Trade Goods
Code Logic: The BusinessZakat struct aggregates cash_on_hand, inventory_value, and receivables.
Fiqh Analysis:
The obligation of Zakat on trade goods is established by the vast majority of scholars (Jumhur), derived from the general Quranic command to spend from "the best of what you have earned."
Primary Evidence (Dalil):
The specific textual proof for valuing merchandise is the Hadith narrated by Samurah bin Jundub (ra):"The Messenger of Allah (ﷺ) used to command us to pay the Sadaqah (Zakat) from what we prepared for sale."
— Sunan Abu Dawud (Hadith 1562).11
Commentary: While the chain of this Hadith has been discussed, its meaning is supported by the practice of the Caliphs, specifically 'Umar ibn al-Khattab (ra), who ordered the valuation of leather and trade goods for Zakat purposes.
3.2 Valuation and Receivables
Code Logic: The library expects inventory_value as an input.
Fiqh Guidance for the User:
The user must be instructed (via documentation) to input the Current Market Value (Qimah Suqiyyah) of the inventory at the time of Zakat calculation, not the cost price. This aligns with the phrase "prepared for sale"—the value is what it would fetch in the market today.
Receivables (Dayn): The inclusion of receivables in the calculation assumes they are Dayn Marjuww (hoped-for debts/good debts). If the debt is doubtful (bad debt), the Maliki view is to pay only upon receipt, while the Shafi'i view requires payment for all back years upon receipt. The library’s inclusion of this field by default aligns with the view of stronger financial caution (Ahwat).
3.3 Deducting Liabilities (Dayn al-Hal)
Code Logic: The formula deducts liabilities_due_now.
Fiqh Analysis:
This logic touches upon the debate: "Does debt prevent Zakat?" (Hal al-dayn yamna' al-Zakat?).
Hanafi View: Debt prevents Zakat on non-apparent wealth (cash/business). If one has 1000 and owes 1000, they have no net wealth.
Shafi'i View: Debt does not prevent Zakat. The owner of the wealth must pay Zakat on what they possess, regardless of what they owe.
Contemporary Standard (AAOIFI/IIFA): Most modern Fatwas adopt a middle ground to suit modern financial structures. They distinguish between "liabilities due immediately" (Dayn al-Hal) and long-term debt (e.g., 20-year mortgages).
Reference: AAOIFI Sharia Standard No. 35 permits the deduction of debts related to the acquisition of commercial assets. The widely accepted Fatwa allows deducting only the upcoming year's installments of long-term debt, rather than the entire principal, to ensure the Zakat base is not completely eroded for wealthy individuals with long-term leverage.18
Classical Support: Ibn Qudamah in Al-Mughni discusses that debt prevents Zakat only when it consumes the Nisab and the debtor has no surplus to pay it.20
Architectural Validation: By specifically naming the field liabilities_due_now (rather than total_liabilities), the library enforces the correct Fiqh interpretation that only immediate/short-term obligations should be deducted, preventing the user from incorrectly deducting long-term non-commercial debts that would invalidate their Zakat liability.
4. Agriculture: The Harvest Tax (Zakat al-Zuru')
Module: src/maal/agriculture.rs
This module implements Zakat on crops, distinguishing between irrigation methods and applying the correct rates.
4.1 Rates Based on Irrigation Effort
Code Logic: The library applies varying rates based on the IrrigationMethod:
Rain (Natural) -> 10% (Ushr)
Irrigated (Artificial/Costly) -> 5% (Half-Ushr)
Mixed -> 7.5% (Three-quarters of Ushr)
Fiqh Analysis:
These rates are textually fixed (Nass) by the Prophet (ﷺ) and are not subject to Ijtihad. The logic reflects the principle that "Cost reduces the obligation."
Primary Evidence (Dalil):"On a land irrigated by rain water or by natural water channels or if the land is wet due to a nearby water channel, Ushr (one-tenth) is compulsory; and on the land irrigated by the well, half of an Ushr (one-twentieth) is compulsory."
— Sahih Bukhari (Hadith 1483); Sahih Muslim (Hadith 981).21
Mixed Irrigation: The 7.5% rate is derived by the Fuqaha (Jurists) for cases where land is watered equally by rain and labor, taking the average of the two rates.22
4.2 Nisab: The Five Awsuq
Code Logic: The library uses a default Nisab of 653 kg.
Fiqh Analysis:
Primary Evidence:"There is no Zakat on grains or dates until such items weigh five Wasqs."
— Sahih Muslim (Hadith 979).24
Conversion (Tahqiq):
1 Wasq = 60 Sa'.
5 Awsuq = 300 Sa'.
1 Sa' = 4 Mudd (Prophetic handfuls).
The conversion to kilograms depends on the specific crop (wheat vs. dates vs. barley) as Sa' is a measure of volume, not weight.
Yusuf Al-Qaradawi's Calculation: In his magnum opus Fiqh al-Zakah, Dr. Qaradawi investigates the weight of a standard Sa' of wheat and concludes that the Nisab is approximately 653 kg.26
Other Views: Some scholars (e.g., Bin Baz) calculate it closer to 612 kg.24
Validation: The library’s choice of 653 kg aligns with the view of Qaradawi, which is widely adopted in modern Zakat standards as a precautionary (Ahwat) threshold. Using a configurable ZakatConfig allows users to adjust this if they follow the 612 kg opinion.
5. Livestock: The Grazing Herds (An'am)
Module: src/maal/livestock.rs
The calculation of Zakat on livestock is unique as it follows a tiered, non-linear step function rather than a flat percentage.
5.1 The Camel Zakat Table
Code Logic: The library implements specific tiers to determine the "Heads Due" (e.g., Bint Makhad, Bint Labun).
Fiqh Analysis:
The tiers for camel Zakat are explicitly detailed in the famous letter of instructions sent by the first Caliph, Abu Bakr As-Siddiq (ra), to his Zakat collectors in Bahrain.
Primary Evidence (Dalil):Narrated Anas (ra): When Abu Bakr sent me to Bahrain... he wrote: "For 25 to 35 camels, one Bint Makhad (1-year-old she-camel)... for 36 to 45, one Bint Labun (2-year-old)... for 46 to 60, one Hiqqah (3-year-old)... for 61 to 75, one Jaza'ah (4-year-old)..."
— Sahih Bukhari (Hadith 1454).28
Validation: The library’s internal logic tables must strictly map to these definitions. The implementation of PaymentPayload::Livestock containing descriptions like "Bint Makhad" is textually compliant and preserves the authentic terminology of the Sunnah.
5.2 The Grazing Condition (Saimah)
Code Logic: The LivestockAssets struct includes a grazing_method. If the method is not Saimah, the Zakat due is zero.
Fiqh Analysis:
According to the majority of scholars (Shafi'i, Hanafi, Hanbali), Zakat on livestock is conditional on the animals being Saimah—freely grazing on public pasture for the majority of the year. If they are Maalufah (fodder-fed/farmed), no Zakat is due on the animals themselves (though they may be subject to Business Zakat if raised for trade).
Primary Evidence:"In the grazing (Saimah) sheep..."
— Sahih Bukhari.
The qualification "grazing" restricts the general ruling (Taqyid al-Mutlaq).
Divergence: The Maliki school charges Zakat on livestock regardless of grazing. However, the library follows the majority view (Jumhur), which is standard for general Zakat calculators unless a "Maliki Strategy" is specifically invoked.30
6. Modern Assets: Income & Investments
Module: src/maal/income.rs & investments.rs
This section addresses Nawazil (contemporary issues) where classical texts provide principles rather than direct rulings.
6.1 Professional Income (Zakat al-Mustafad)
Code Logic: The library allows calculations via Gross (total income) or Net (income minus expenses) methods.
Fiqh Analysis:
The Concept: This pertains to Mal Mustafad (wealth acquired during the year). Classical positions varied: some required waiting for the Hawl (year) to pass on every paycheck, while others allowed paying immediately.
Modern Ijtihad (Qaradawi): In Fiqh al-Zakah, Dr. Yusuf Al-Qaradawi argues that modern salaries and wages should be treated analogously to agricultural harvest—Zakat is due immediately upon receipt, without waiting for a year, to ensure the poor are not deprived of the wealth of high-earning professionals.
Calculation Methods: Qaradawi proposes two analogies:
Gross: Pay Zakat on the total earnings immediately (analogous to the 10% or 5% of crops, though usually applied at 2.5% for money).
Net: Deduct basic needs (Hajah Asliyyah) and debts, then pay on the surplus.
Validation: The library’s support for both Gross and Net methods perfectly accommodates this modern spectrum of opinion, allowing users to follow the stricter (Gross) or more lenient (Net) Fatwa.32
6.2 Stocks and Cryptocurrencies
Code Logic: InvestmentAssets calculates 2.5% on the market value if the asset type implies trading.
Fiqh Analysis:
Classification: Stocks (Ashum) and Crypto are classified by the majority of modern councils (AAOIFI, IIFA) as Urud al-Tijarah (Trade Goods) if held for capital appreciation.
Standards:
AAOIFI Sharia Standard No. 35 (Zakah): States that for shares acquired for trading purposes, Zakat is paid on the market value. For shares held for long-term income (dividends), Zakat is paid only on the Zakatable Assets of the company (cash, receivables, inventory).33
IIFA Resolutions: Crypto assets, when recognized as wealth (Mal), are subject to Zakat at 2.5% of their market value if they reach the Nisab.35
Validation: The library’s logic of treating InvestmentType::Crypto and InvestmentType::Stock as zakatable at market value is compliant with the "Trading" classification, which is the default assumption for individual investors in most Zakat applications.
7. Portfolio & Aggregation: The Logic of Unity
Module: src/portfolio.rs
The ZakatPortfolio module implements the advanced logic of aggregating diverse asset classes.
7.1 Dam' al-Amwal (Wealth Aggregation)
Code Logic: The library converts Gold, Silver, Cash, Stocks, and Business Assets into a single monetary baseline. It then checks if Total Value > Monetary Nisab. If yes, Zakat is payable on all assets, even if an individual asset (e.g., Gold) is below the 85g threshold.
Fiqh Analysis:
This implements the principle of Dam' al-Amwal (Joining Wealth).
The Issue: A person has 50g of Gold (below Nisab) and $3,000 Cash (below Nisab). Separately, they are exempt. Combined, they exceed the threshold.
Hanafi View (Adopted): The Hanafi school explicitly mandates combining Gold, Silver, and Commercial Goods by value to complete the Nisab. They argue that these assets share the same Illah (effective cause) of "Absolute Value" (Thamaniyyah).
Citation: "If he has half Nisab of gold and half of silver, he pays Zakat... for they are of the same genus regarding the objective of price" (Al-Hidayah).37
Shafi'i View: Generally requires Gold to complete Gold, and Silver to complete Silver, without combining them.
Validation: The library’s aggregation logic prioritizes the view that is Anfa' lil-fuqara (more beneficial for the poor) and prevents the artificial splitting of wealth to escape Zakat. By adopting the Hanafi/Majority aggregation method, the library ensures strict compliance for the widest user base.
8. Madhab Differences & The "Lower of Two"
Module: src/madhab.rs
Code Logic: The NisabStandard enum supports Gold, Silver, or LowerOfTwo.
Fiqh Analysis:
Lower of Two (Adna al-Nisabayn): This is the standard Hanafi position. In the time of the Prophet (ﷺ), 20 Dinars (Gold) and 200 Dirhams (Silver) had roughly equal purchasing power (1:10 ratio). Today, the silver Nisab (~$400) is drastically lower than the gold Nisab (~$6,000).
Rationale: Using the Silver standard (Lower of Two) forces many more people to pay Zakat, which increases the funds available for the poor. Hanafis argue this is preferred because it is safer for the payer (Ahwat) and better for the recipient.
Gold Standard: Many contemporary scholars (like Qaradawi) argue for the Gold Standard because the Silver Nisab is now so low it captures people who are essentially poor.
Validation: The library’s configuration option allows the user to choose. The default to LowerOfTwo (if set) aligns with the conservative Hanafi view, while offering Gold supports the modern socio-economic adjustment advocated by Qaradawi.39
9. Code Documentation & Citations
The following comment blocks are prepared for insertion into the zakatrs source code to document the Fiqh compliance of each module.
src/maal/precious_metals.rs

Rust


//! # Fiqh Compliance: Precious Metals
//!
//! ## Nisab (Threshold)
//! - **Gold**: 20 Dinars (approx. 85 grams).
//! - **Silver**: 200 Dirhams (approx. 595 grams).
//! - **Source**: Sunan Abu Dawud (1573) and Sahih Muslim (979).
//!
//! ## Jewelry Exemption (Huliyy al-Mubah)
//! This module supports divergent Madhab views via `ZakatStrategy`:
//! - **Shafi'i/Maliki/Hanbali**: Personal permissible jewelry is **EXEMPT** (Reference: *Al-Majmu'* by Al-Nawawi, *Al-Mughni* by Ibn Qudamah).
//! - **Hanafi**: Personal jewelry is **ZAKATABLE** (Reference: *Al-Hidayah* by Al-Marghinani, *Bada'i al-Sana'i* by Al-Kasani).
//!
//! ## Purity Logic
//! - Zakat is due on the *pure* metal content.
//! - Logic: `weight * (karat / 24)` extracts the zakatable 24K equivalent.


src/maal/business.rs

Rust


//! # Fiqh Compliance: Business Assets (Urud al-Tijarah)
//!
//! ## Obligation
//! - Based on the Hadith of Samurah bin Jundub: "The Prophet (ﷺ) commanded us to pay Zakat from what we prepared for sale." (Sunan Abu Dawud 1562).
//!
//! ## Valuation Logic
//! - **Formula**: `(Cash + Market Value of Inventory + Good Receivables) - Immediate Liabilities`.
//! - **Valuation**: Inventory must be valued at current *Market Price* at the time of Zakat, not Cost Price.
//! - **Debts**: Deducting `liabilities_due_now` aligns with the principle of *Dayn al-Hal* (immediate debt) preventing Zakat, as supported by AAOIFI Standard 35.


src/maal/agriculture.rs

Rust


//! # Fiqh Compliance: Agriculture
//!
//! ## Rates
//! - **10% (Ushr)**: Rain-fed/Natural irrigation. (Source: Sahih Bukhari 1483).
//! - **5% (Half-Ushr)**: Irrigated/Labor-intensive. (Source: Sahih Muslim 981).
//! - **7.5%**: Mixed irrigation methods (derived via Ijtihad).
//!
//! ## Nisab
//! - **Threshold**: 5 Awsuq. (Source: Sahih Muslim 979).
//! - **Conversion**: Configurable, defaults to **653 kg** based on the research of Dr. Yusuf Al-Qaradawi (*Fiqh al-Zakah*).


src/maal/livestock.rs

Rust


//! # Fiqh Compliance: Livestock
//!
//! ## Logic
//! - Implements the specific camel age tiers (Bint Makhad, Bint Labun, Hiqqah, Jaza'ah) as defined in the **Letter of Abu Bakr (ra)** (Sahih Bukhari 1454).
//!
//! ## Conditions
//! - **Saimah**: Zakat is only calculated if `grazing_method` is Natural/Saimah, adhering to the majority view (Jumhur) that fodder-fed animals are exempt from Livestock Zakat.


src/portfolio.rs

Rust


//! # Fiqh Compliance: Portfolio Aggregation
//!
//! ## Principle: Dam' al-Amwal (Joining Wealth)
//! - Implements the **Hanafi** and Majority view that Gold, Silver, Cash, and Trade Goods are of a single genus (*Thamaniyyah*) and must be combined to reach the Nisab.
//! - **Benefit**: This ensures the poor receive their due from wealth that would otherwise be exempt if split (*Anfa' lil-fuqara*).


10. Conclusion
The zakatrs library demonstrates a high degree of fidelity to Islamic Jurisprudence. Its architecture—specifically the use of rust_decimal for precision and the ZakatStrategy trait for handling Ikhtilaf—is structurally sound and religiously compliant. By accommodating the nuances of the four Madhabs and integrating modern standards from AAOIFI, the library is suitable for use by a broad spectrum of the Muslim community, providing a tool that is both technically robust and spiritually safe (Halal).
Allahu A'lam (Allah knows best).
README.md Section: Fiqh References
📜 Fiqh References & Compliance
This library is built upon rigorous research into Islamic Jurisprudence (Fiqh). Below are the primary sources (Dalil) and scholarly opinions supporting the calculation logic.
1. Precious Metals (Gold & Silver)
Nisab: 85g Gold (20 Dinars) and 595g Silver (200 Dirhams).
Source: Hadith of Ali (ra): "Nothing is incumbent on you... till it reaches twenty dinars." (Sunan Abu Dawud 1573).
Source: "No Zakat on less than 5 Awaq (of silver)." (Sahih Muslim 979).
Jewelry Logic:
Hanafi (Default): Personal jewelry is Zakatable. Reference: Al-Hidayah. Based on the Hadith of the woman with bracelets of fire (Abu Dawud 1558).
Shafi'i/Maliki/Hanbali (Optional): Personal jewelry is Exempt. Reference: Al-Majmu'. Based on the practice of Aisha (ra) (Muwatta Malik).
2. Business Assets (Urud al-Tijarah)
Basis: Trade goods are valued at market price.
Source: Hadith of Samurah bin Jundub: "The Prophet (ﷺ) commanded us to pay Zakat from what we prepared for sale." (Sunan Abu Dawud 1562).
Debts: Logic supports deducting immediate liabilities (Dayn al-Hal) before calculation, aligning with classical views in Al-Mughni and AAOIFI standards.
3. Agriculture (Zuru' wal-Thimar)
Rates:
10% (Ushr): Rain-fed. Source: Sahih Bukhari 1483.
5% (Half-Ushr): Irrigated (cost/labor). Source: Sahih Muslim 981.
Nisab: 5 Awsuq (~653 kg).
Source: "No Zakat on grains... until they weigh five Wasqs." (Sahih Muslim 979).
Conversion: ~653kg based on Dr. Yusuf Al-Qaradawi's calculation in Fiqh al-Zakah.
4. Livestock (An'am)
Camels: Implements the specific age tiers (Bint Makhad, Bint Labun, etc.).
Source: The Letter of Abu Bakr (ra) outlining Zakat rules (Sahih Bukhari 1454).
Grazing: Logic enforces the Saimah (naturally grazing) condition.
5. Modern Assets & Portfolio
Income: Supports the "Net vs Gross" opinions for Zakat al-Mustafad (Qaradawi).
Stocks/Crypto: Classified as Trade Goods (Urud) subject to 2.5% on market value (AAOIFI Standard 35).
Aggregation (Dam' al-Amwal): Implements the Hanafi/Majority view of combining Gold, Silver, and Cash to reach Nisab, ensuring maximum benefit for the poor (Anfa' lil-fuqara).
Works cited
Sunan Abi Dawud 1573 - Zakat (Kitab Al-Zakat) - كتاب الزكاة - Sunnah.com - Sayings and Teachings of Prophet Muhammad (صلى الله عليه و سلم), accessed December 30, 2025, https://sunnah.com/abudawud:1573
Obligatory Charity Tax (Zakat) - Sunnah.com - Sayings and Teachings of Prophet Muhammad (صلى الله عليه و سلم), accessed December 30, 2025, https://sunnah.com/bukhari/24
Fiqh-us-Sunnah Hadith 3.14 | Volume 3 - Alim.org, accessed December 30, 2025, https://www.alim.org/hadith/fiqh-us-sunnah/3/14/
Zakat on gold and silver - articles - Islamic Fiqh, accessed December 30, 2025, https://islamicfiqh.net/en/articles/zakat-on-gold-and-silver-75
Do I Have To Pay Zakat For The Gold Which I Possess? - AMJA Online, accessed December 30, 2025, https://www.amjaonline.org/fatwa/en/78921/do-i-have-to-pay-zakat-for-the-gold-which-i-possess
Zakat Calculator - Minhaj Welfare Foundation, accessed December 30, 2025, https://minhajwelfare.org/zakat-calculator/
Zakat on gold jewellery (English) - Majlis Ugama Islam Singapura, accessed December 30, 2025, https://www.muis.gov.sg/resources/khutbah-and-religious-advice/fatwa/zakat-on-gold-jewellery--english/
Calculation of Zakat on gold | Qatar Charity Blog, accessed December 30, 2025, https://www.qcharity.org/blog/20557/calculation-of-zakat-on-gold?lang=en
Zakat on Jewelry - NZF Canada, accessed December 30, 2025, https://www.nzfcanada.com/research/zakat-on-jewelry
Akhsar al-Mukhtasarat | Book of Zakat| Gold, Silver and Trade Goods | Mohammad Zahid - Ink of Faith, accessed December 30, 2025, https://www.inkoffaith.com/post/akhsar-al-mukhtasarat-book-of-zakat-gold-silver-and-trade-goods
SUNAN ABU-DAWUD, Book 3: Zakat (Kitab Al-Zakat), accessed December 30, 2025, https://www.iium.edu.my/deed/hadith/abudawood/003_sat.html
Zakah on jewellery that has been prepared for use - Islam Question & Answer, accessed December 30, 2025, https://islamqa.info/en/answers/59866
Zakat on Worn Gold: Its Quorum, Islamic Rulings, and Legal Perspective - مبادرة مسارات, accessed December 30, 2025, https://masarat-sy.org/en/zakat-on-worn-gold-its-quorum-islamic-rulings-and-legal-perspective/
Al-Hidayah, accessed December 30, 2025, https://ia600505.us.archive.org/15/items/Hedaya_201703/Hedaya.pdf
How to Calculate Zakah on Gold - Islam Question & Answer, accessed December 30, 2025, https://islamqa.info/en/answers/214221
Zakat from Online Businesses Brings Blessings from Every Sale - Dompet Dhuafa, accessed December 30, 2025, https://www.dompetdhuafa.org/en/zakat-from-online-businesses-brings-blessings-from-every-sale/
Zakat (Kitab Al-Zakat) - Sunnah.com - Sayings and Teachings of Prophet Muhammad (صلى الله عليه و سلم), accessed December 30, 2025, https://sunnah.com/abudawud/9
Are Non-Current Liabilities Deductible in Zakat Calculation? - Pakistan Sweet Home, accessed December 30, 2025, https://www.pakistansweethome.org.pk/zakat/can-non-current-liabilities-be-deducted-for-business-zakat
Debts and liabilities in the context of paying Zakat - NZF, accessed December 30, 2025, https://nzf.org.uk/knowledge/payment-of-zakat-deductible-liabilities/
Deducting the Value of Debt from the Nisaab When Paying Zakaah - إسلام ويب, accessed December 30, 2025, https://www.islamweb.net/en/fatwa/426541/deducting-the-value-of-debt-from-the-nisaab-when-paying-zakaah
Hadith on Obligatory Charity Tax (Zakat): On A Land Irrigated By Rain Water Or By Natural Water Channels Or If The Land Is Wet Due To A Near By Water Channel Ushr (Ie One-Tenth) Is Compulsory (As Zakat); And On The Land Irrigated By The Well, Half Of An Ushr (Ie One- - IslamiCity, accessed December 30, 2025, https://www.islamicity.org/hadith/search/index.php?q=1422&sss=1
Zakah on Agricultural Products - Islamic Heritage Center, accessed December 30, 2025, https://www.ihcproject.com/agricultural-zakah
Here's How to Calculate Corn Farming Zakat, Let's Check It Out! - Dompet Dhuafa, accessed December 30, 2025, https://www.dompetdhuafa.org/en/heres-how-to-calculate-corn-farming-zakat-lets-check-it-out/
Understanding Zakat on Agricultural Produce: Fruits and Grains - Pakistan Sweet Home, accessed December 30, 2025, https://www.pakistansweethome.org.pk/blog/zakat/zakat-on-agricultural-produce
Zakah on agricultural produce - Islam Question & Answer, accessed December 30, 2025, https://islamqa.info/en/answers/172973
(PDF) Zakah on Agriculture Reformation: An Analysis in Malaysia - ResearchGate, accessed December 30, 2025, https://www.researchgate.net/publication/274634619_Zakah_on_Agriculture_Reformation_An_Analysis_in_Malaysia
41st inaugural lecture olabisi onabanjo university ago-iwoye, accessed December 30, 2025, https://oouagoiwoye.edu.ng/inaugural_lectures/Islam%20and%20World%20Peace.pdf
Sahih Bukhari Volume 2, Book 24, Hadith Number 528., accessed December 30, 2025, https://hadithcollection.com/sahihbukhari/sahih-bukhari-book-24-obligatory-charity-tax-zakat/sahih-bukhari-volume-002-book-024-hadith-number-528
Sahih al-Bukhari 1454 - Obligatory Charity Tax (Zakat) - كتاب الزكاة - Sunnah.com - Sayings and Teachings of Prophet Muhammad (صلى الله عليه و سلم), accessed December 30, 2025, https://sunnah.com/bukhari:1454
Fiqh-us-Sunnah Hadith 3.39 | Volume 3 - Alim.org, accessed December 30, 2025, https://www.alim.org/hadith/fiqh-us-sunnah/3/39/
Fiqh-us-Sunnah, Volume 3: Zakah on Animals - Islamicstudies.info, accessed December 30, 2025, https://www.islamicstudies.info/subjects/fiqh/fiqh_us_sunnah/fus3_43.html
Zakat Calculation : Based on Fiqh-uz-Zakat by Yusuf al-Qaradawi 9780860375678, accessed December 30, 2025, https://dokumen.pub/zakat-calculation-based-on-fiqh-uz-zakat-by-yusuf-al-qaradawi-9780860375678.html
How to Pay Zakat on Stocks - Muslim Xchange, accessed December 30, 2025, https://muslimxchange.com/insights/how-to-pay-zakat-on-stocks/
Analysing the AAOIFI Sharīʿah standard on zakat - Emerald Publishing, accessed December 30, 2025, https://www.emerald.com/insight/content/doi/10.1108/jmlc-10-2020-0117/full/pdf?title=analysing-the-aaoifi-shariah-standard-on-zakat
Zakat payment from cryptocurrencies and crypto assets - MUZAKARAH CENDEKIAWAN SYARIAH NUSANTARA KE-19, accessed December 30, 2025, https://muzakarah.inceif.edu.my/kertas-kerja-slide/bacaan-tambahan/Zakat_payment_from_cryptocurrencies_and_crypto_assets_Aishath%20Muneeza_Magda%20Ismail.pdf
Crypto Assets: Zakat of the Digital World - Shariyah Review Bureau, accessed December 30, 2025, https://shariyah.net/wp-content/uploads/2023/05/Crypto-Assets-Zakat-of-the-Digital-World.pdf
Zakat Ruling: Combining Gold and Currency to Complete Nisab | EN.tohed.com, accessed December 30, 2025, https://en.tohed.com/threads/zakat-ruling-combining-gold-and-currency-to-complete-nisab.5901/
Can cash be added to gold and silver to complete the nisaab? - Islam Question & Answer, accessed December 30, 2025, https://islamqa.info/en/answers/201807
Nisab for Zakat on Gold and Silver, what is it? - joebradford.net, accessed December 30, 2025, https://joebradford.net/nisab-for-zakat-on-gold-and-silver/
