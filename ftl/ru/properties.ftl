abbreviation = аббревиатура
common_name = общее название
edit = изменить
experimental = экспериментальное значение
fatty_acid_mass = { mass } { -fatty_acid_term(genus: "genitive") }
formula = формула
mass = масса
methyl_ester_mass = { mass } метилового эфира
molar_mass = молярная { mass }
names = названия
names_description = показать устоявшиеся названия для жирных кислот
properties = свойства
properties_description = показать { properties }
resize = изменить размер
selectivity_factor = фактор селективности
species = вид
systematic_name = систематическое название
theoretical = теоретическое значение

fatty_acid = { -fatty_acid_term }
    .abbreviation = ЖК
triacylglycerol = триацилглицерин
    .abbreviation = ТАГ
diacylglycerol = диацилглицерин
    .abbreviation = ДАГ
monoacylglycerol = моноацилглицерин
    .abbreviation = МАГ

configuration = конфигурация
calculation = вычисление
composition = композиция

# Central panel

# Left panel
precision = точность
percent = проценты

## Calculation
fraction = доля
as_is = как есть
to_mass_fraction = в массовую долю
to_mole_fraction = в мольную долю
sign = знак
signed = со знаком
    .description = теоретически рассчитанные отрицательные значения отстаются без изменения
unsigned = без знака
    .description = теоретически рассчитанные отрицательные значения замещаются нулем
from = вычислить из
    .description = вычислить значения 1,3-{ diacylglycerol.abbreviation } из
from_dag = из 1,2/2,3-{ diacylglycerol.abbreviation }
    .description = вычислить значения 1,3-{ diacylglycerol.abbreviation } из 1,2/2,3-{ diacylglycerol.abbreviation }
from_mag = из 2-{ monoacylglycerol.abbreviation }
    .description = вычислить значения 1,3-{ diacylglycerol.abbreviation } из 2-{ monoacylglycerol.abbreviation }

## Composition
adduct = аддукт
method = метод
gunstone = Ганстоун
    .description = вычисление по теории Ганстоуна
vander_wal = Вандер Валь
    .description = вычисление по теории Вандер Валя
group = группировка
sort = сотрировка
by_key = по ключу
    .description = сортировать по ключу
by_value = по значению
    .description = сортировать по значению
order = порядок
ascending = по возрастанию
    .description = обратный порядок (от максимума к минимуму)
descending = по убыванию
    .description = прямой порядок (от минимума к максимуму)

key = ключ
value = значение

-fatty_acid_term = { $genus ->
   *[nominative] жирная кислота
    [genitive] жирной кислоты
}
