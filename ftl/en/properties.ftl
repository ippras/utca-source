properties = Properties
    .description = Show properties

fatty_acid_mass = { -fatty_acid_term } { mass }
formula = formula
mass = mass
methyl_ester_mass = methyl ester { mass }
molar_mass = molar { mass }
species = species

experimental = experimental
    .title_case = Experimental
factors = factors
    .title_case = Factors
identifier = identifier
    .abbreviation = ID
    .title_case = Identifier
index = index
    .title_case = Index
label = label
    .title_case = Label
theoretical = theoretical
    .title_case = Theoretical

fatty_acid = { -fatty_acid_term }
    .abbreviation = FA
    .title_case = Fatty acid
triacylglycerol = triacylglycerol
    .abbreviation = TAG
    .title_case = Triacylglycerol
diacylglycerol = diacylglycerol
    .abbreviation = DAG
    .title_case = Diacylglycerol
monoacylglycerol = monoacylglycerol
    .abbreviation = MAG
    .title_case = Monoacylglycerol
enrichment_factor = enrichment factor
    .abbreviation = EF
    .title_case = Enrichment factor
selectivity_factor = selectivity factor
    .abbreviation = SF
    .title_case = Selectivity factor

equivalent_carbon_number_composition = equivalent carbon number composition
    .abbreviation = NC
positional_equivalent_carbon_number_composition = positional equivalent carbon number composition
    .abbreviation = PNC
stereo_equivalent_carbon_number_composition = stereo equivalent carbon number composition
    .abbreviation = SNC
mass_composition = mass composition
    .abbreviation = MC
positional_mass_composition = positional mass composition
    .abbreviation = PMC
stereo_mass_composition = stereo mass composition
    .abbreviation = SMC
type_composition = type composition
    .abbreviation = TC
positional_type_composition = positional type composition
    .abbreviation = PTC
stereo_type_composition = stereo type composition
    .abbreviation = STC
species_composition = species composition
    .abbreviation = SC
positional_species_composition = positional species composition
    .abbreviation = PSC
stereo_species_composition = stereo species composition
    .abbreviation = SSC

calculate = calculate
calculation = calculation
compose = compose
composition = composition
configuration = configuration
configure = configure

# Central panel

# Left panel
precision = precision
percent = percent

## Calculation
fraction = fraction
as_is = as is
to_mass_fraction = to mass fraction
to_mole_fraction = to mole fraction
sign = sign
signed = signed
    .description = theoretically calculated negative values are as is
unsigned = unsigned
    .description = theoretically calculated negative values are replaced with zeros
from = calculate
    .description = calculate 1,3-{ diacylglycerol.abbreviation }s
from_dag = from 1,2/2,3-{ diacylglycerol.abbreviation }s
    .description = calculate 1,3-{ diacylglycerol.abbreviation }s from 1,2/2,3-{ diacylglycerol.abbreviation }s
from_mag = from 2-{ monoacylglycerol.abbreviation }s
    .description = calculate 1,3-{ diacylglycerol.abbreviation }s from 2-{ monoacylglycerol.abbreviation }s

## Composition
adduct = adduct
method = method
gunstone = Gunstone
    .description = calculate by { gunstone }'s theory
vander_wal = Vander Wal
    .description = calculate by { vander_wal }'s theory
group = group
sort = sort
by_key = key
    .description = sort by key
by_value = value
    .description = sort by value
order = order
ascending = ascending
    .description = direct order (from min to max)
descending = descending
    .description = reverse order (from max to min)

key = key
value = value

-fatty_acid_term = fatty acid
