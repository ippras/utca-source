properties = Свойства
    .description = Показать свойства

fatty_acid_mass = { mass } { -fatty_acid_term(genus: "genitive") }
formula = формула
mass = масса
methyl_ester_mass = { mass } метилового эфира
molar_mass = молярная { mass }
species = вид

experimental = эксперимент
    .title_case = Эксперимент
factors = факторы
    .title_case = Факторы
identifier = идентификатор
    .abbreviation = ИД
    .title_case = Идентификатор
index = индекс
    .title_case = Индекс
label = метка
    .title_case = Метка
theoretical = расчеты
    .title_case = Расчеты

fatty_acid = { -fatty_acid_term }
    .abbreviation = ЖК
    .title_case = Жирная кислота
triacylglycerol = триацилглицерин
    .abbreviation = ТАГ
    .title_case = Триацилглицерин
diacylglycerol = { $genus ->
    *[nominative] диацилглицерин
    [genitive] диацилглицерина
}
    .abbreviation = ДАГ
    .title_case = Диацилглицерин
monoacylglycerol = { $genus ->
    *[nominative] моноацилглицерин
    [genitive] моноацилглицерина
}
    .abbreviation = МАГ
    .title_case = Моноацилглицерин
enrichment_factor = фактор обогощения
    .abbreviation = ФО
    .title_case = Фактор обогощения
selectivity_factor = фактор селективности
    .abbreviation = ФС
    .title_case = Фактор селективности

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
